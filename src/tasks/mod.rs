use daggy::Dag;
use daggy::petgraph::visit::Bfs;
use daggy::petgraph::graph::NodeIndex;
use url::Url;

use download::{self, DownloadResult};
use parser;
use storage;
pub use self::resource::{Stage, DetailsStorage, Resource};


mod resource;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Running,
    Done,
}


pub struct TaskContext {
    dag: Dag<Resource, ()>,
    roots: Vec<NodeIndex>,
    download_pool: download::Pool,
}

impl TaskContext {
    pub fn new(websites: &[String], thread_count: u32) -> TaskContext {
        let mut dag = Dag::new();
        let roots = websites.iter()
                            .map(|website| {
                                dag.add_node(Resource::new(Url::parse(website).unwrap()))
                            })
                            .collect();

        let mut pool = download::Pool::new(thread_count);
        pool.start();

        TaskContext {
            dag: dag,
            roots: roots,
            download_pool: pool,
        }
    }

    #[allow(dead_code)]
    pub fn active_download_count(&self) -> i32 {
        let mut count = 0;
        let graph = self.dag.graph();

        for root in &self.roots {
            let mut bfs = Bfs::new(graph, *root);
            while let Some(n) = bfs.next(graph) {
                if self.dag[n].get_stage() == Stage::Download {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn process(&mut self) -> State {
        // Process latest download results
        let download_results = self.download_pool.get_results();

        for result in download_results {
            let resource = &mut self.dag[result.idx];

            match result.result {
                DownloadResult::Success { response, time } => resource.downloaded(response, time),
                DownloadResult::Skip => resource.skip(),
                DownloadResult::Retry => resource.retry_download(),
                DownloadResult::Failed => resource.failed(),
            }
        }

        // Update queues
        let mut done = true;
        let mut waiting = Vec::new();
        let mut open = Vec::new();

        {
            let graph = self.dag.graph();
            for root in &self.roots {
                let mut bfs = Bfs::new(graph, *root);
                while let Some(n) = bfs.next(graph) {
                    let r = &self.dag[n];

                    // This is just a heuristic as the node might get finished
                    // during this iteration of process_next(). But it's good enough.
                    if r.get_stage() != Stage::Done && !r.is_failed() {
                        done = false;
                    }

                    if r.get_stage() == Stage::Waiting {
                        waiting.push(n);
                    } else if !is_idle_stage(r.get_stage()) && !r.is_failed() {
                        open.push(n);
                    }
                }
            }
        }

        waiting.sort();
        waiting.dedup();

        open.sort();
        open.dedup();

        // Start downloading all waiting resources
        while let Some(n) = waiting.pop() {
            self.dispatch_task(n);
        }

        // Process open tasks
        while let Some(n) = open.pop() {
            self.dispatch_task(n);
        }

        if done {
            State::Done
        } else {
            State::Running
        }
    }

    fn find_node<F: Fn(&Resource) -> bool>(&self, f: F) -> Option<NodeIndex> {
        let mut node = None;
        let graph = self.dag.graph();
        for root in &self.roots {
            let mut bfs = Bfs::new(graph, *root);
            while let Some(n) = bfs.next(graph) {
                if f(&self.dag[n]) {
                    node = Some(n);
                }
            }
        }

        node
    }

    fn dispatch_task(&mut self, node: NodeIndex) {
        let stage = self.dag[node].get_stage();

        trace!("Processing node {:?} with stage {:?}", node, stage);
        // trace!("Number of active downloads: {}",
        //       self.active_download_count());

        match stage {
            Stage::Waiting => {
                let item = &mut self.dag[node];
                item.start_download();
                self.download_pool.add(node, item);
            }
            Stage::Download => panic!("Cannot dispatch task with stage = Download"),
            Stage::Parse => {
                let mut explored_resources = parser::task_parse_resource(&mut self.dag[node]);
                explored_resources.sort();
                explored_resources.dedup();  // Note: Doesn't work without sort

                for explored in explored_resources {
                    // See if explored resource is already present in DAG...
                    if let Some(n) = self.find_node(|n| *n.get_url() == explored) {
                        self.dag.add_edge(node, n, ()).unwrap();
                    } else {
                        self.dag.add_child(node, (), Resource::new(explored));
                    }
                }
            }
            Stage::Store => storage::store_resource(&mut self.dag[node]),
            Stage::Done => panic!("Cannot dispatch task with stage = Done"),
        }
    }
}


fn is_idle_stage(stage: Stage) -> bool {
    stage == Stage::Done || stage == Stage::Download
}
