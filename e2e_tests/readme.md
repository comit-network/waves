Writing e2e tests is hard. Here are some hints on how to make them as reliable as possible:

- run tests sequentially: in order to rule out random errors, it is recommended to run all tests sequentially.
  For each test we have a separate run target in our package.json.
  If you want to run all tests in sequence, run:

```bash
yarn run all-tests
```

- tests run in headless mode by default.
  If you want to run them in the browser and watch for errors, export the environment variable: `BROWSER=true` before running the tests.
  If you want to run all tests in the browser in sequence, run:

```bash
yarn run all-tests-browser
```

- each test should use its own wallet.
  The setup for this is a bit cumbersome but below is an explanation which can help you to set things up:

To generate a wallet run the `create wallet test` with the following command:

```bash
DEBUG=setup yarn jest -t 'create'
yarn run
```

this will print your wallet data.
With this data you can pre-create a wallet like this which you can use in your own test.

```javascript
const setupTestingWallet = async (
    driver: WebDriver,
    extensionTitle: string,
) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.wallets','demo');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$QmUeLBayg1NxXuMCMmot3Q==$Ogoo7ALW3gkqInRVb6QK/0RrJiQmxzGXpdwdRkl3Ah4=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','8f17fed72c43e5d436fbfd8b185a9cca7f7174d5417750202870d6cecf48528f$75e519b159436a43089420ffedd882338dad57df7ee417af25e60ed06822c61f6ff061a54f046c96e42132cdac28db796d4fc900dcfbee73aeb9cd18d16af3b6c2aab96d04e29916eb3fca2c72d3ca4fc6fa88757dc719dbc87dcef6a8062864d3c70e7bdbedc7a89348a44351182f3211b74a3ad6ebd371b391366a78bd88');",
    );

    return "el1qqfqjvmhpv0lkhzjl8a5nqhz07s23lx84ug9fyj35l8rvm7vf4adqvh3axdppyl8aqmsu46p2qf8zj4zqmq8lxhh5h0gsxl7ym";
};
```

Make sure to extend the `e2e_test_setup.sh` so that your wallet has funding when starting the test.
As a rule of thumb: we don't want to have to call the faucet during our tests as this will add an additional source of randomness.

```bash
curl -POST  http://localhost:3030/api/faucet/el1qqfqjvmhpv0lkhzjl8a5nqhz07s23lx84ug9fyj35l8rvm7vf4adqvh3axdppyl8aqmsu46p2qf8zj4zqmq8lxhh5h0gsxl7ym -d {} > /dev/null
```

Once this is all done, testing should be fun 🥳.
