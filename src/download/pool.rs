use std::sync::mpsc;  // FIXME: Use chan and select! {}
use std::thread;

use chan;
use daggy::petgraph::graph::NodeIndex;
use url::Url;

use download;
use tasks;


#[derive(Debug)]
struct Task {
    idx: NodeIndex,
    url: Url,
    try: i32,
}


#[derive(Debug)]
pub struct Download {
    pub idx: NodeIndex,
    pub result: download::DownloadResult,
}


pub struct DownloadPool {
    task_send: chan::Sender<Task>,
    task_recv: chan::Receiver<Task>,
    results_send: mpsc::Sender<Download>,
    results_recv: mpsc::Receiver<Download>,
    thread_count: u32,
}

impl DownloadPool {
    pub fn new(thread_count: u32) -> DownloadPool {
        let (task_send, task_recv) = chan::async();
        let (results_send, results_recv) = mpsc::channel();

        DownloadPool {
            task_send: task_send,
            task_recv: task_recv,
            results_send: results_send,
            results_recv: results_recv,
            thread_count: thread_count,
        }
    }

    pub fn start(&mut self) {
        debug!("Starting thread pool with {} threads", self.thread_count);

        for tid in 0..self.thread_count {
            let task_recv = self.task_recv.clone();
            let result_send = self.results_send.clone();

            thread::Builder::new()
                .name(format!("download_{}", tid))
                .spawn(move || {
                    while let Some(task) = task_recv.recv() {
                        let task: Task = task;
                        let url = task.url.serialize();

                        debug!("Downloading {} ({:?}) on thread {}", url, task.idx, tid);

                        let result = download::download_url(task.try, task.url);
                        debug!("Finished download of {} ({:?}) on thread {}",
                               url,
                               task.idx,
                               tid);

                        result_send.send(Download {
                                       idx: task.idx,
                                       result: result,
                                   })
                                   .unwrap();
                    }
                })
                .unwrap();
        }
    }

    pub fn add(&mut self, id: NodeIndex, item: &mut tasks::Resource) {
        self.task_send.send(Task {
            idx: id,
            url: item.get_url().clone(), // FIXME: Can we do this without clone?
            try: item.get_retries(),
        });
    }

    pub fn get_results(&mut self) -> Vec<Download> {
        let mut results = Vec::new();

        while let Ok(result) = self.results_recv.try_recv() {
            results.push(result);
        }

        results
    }
}
