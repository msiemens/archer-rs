module.exports = {
    'parser': 'babel-eslint',
    'extends': 'eslint:recommended',
    'rules': {
        'indent': [1, 2, {'SwitchCase': 1}],
        'quotes': [1, 'single'],
        'semi': [2, 'always'],
        'comma-dangle': 0,
        'no-console': 1,
        'no-unused-vars': 1,
        'jsx-quotes': [2, 'prefer-single'],
        'react/jsx-boolean-value': 2,
        'react/jsx-curly-spacing': [2, 'never'],
        'react/jsx-equals-spacing': [2, 'never'],
        'react/jsx-handler-names': 0,
        'react/jsx-key': 2,
        'react/jsx-no-bind': 1,
        'react/jsx-no-duplicate-props': 1,
        'react/jsx-no-undef': 1,
        'react/jsx-pascal-case': 1,
        'react/jsx-uses-react': 1,
        'react/jsx-uses-vars': 1,
        'react/no-danger': 1,
        'react/no-deprecated': 1,
        'react/no-did-mount-set-state': 1,
        'react/no-did-update-set-state': 1,
        'react/no-direct-mutation-state': 1,
        'react/no-is-mounted': 1,
        'react/no-multi-comp': [1, { 'ignoreStateless': true }],
        'react/no-set-state': 1,
        'react/no-string-refs': 1,
        'react/no-unknown-property': 1,
        'react/prefer-es6-class': 1,
        'react/react-in-jsx-scope': 1,
        'react/require-extension': 1,
        'react/self-closing-comp': 2,
        'react/sort-comp': 2,
        'react/wrap-multilines': 2
    },
    'env': {
        'es6': true,
        'browser': true,
        'node': true,
        'jquery': true
    },
    'parserOptions': {
        'ecmaVersion': 6,
        'ecmaFeatures': {
            'jsx': true,
            'experimentalObjectRestSpread': true,
            'classes': true
        },
        'sourceType': 'module'
    },
    'plugins': [
        'react'
    ]
};
