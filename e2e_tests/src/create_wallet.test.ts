import Debug from "debug";
import fetch from "node-fetch";
import { By, until } from "selenium-webdriver";
import { setupBrowserWithExtension, switchToWindow } from "./utils";

const getElementById = async (driver, xpath, timeout = 4000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

describe("create wallet", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    beforeAll(async () => {
        let { driver: d, extensionId: eId, extensionTitle: eTitle, webAppTitle: wTitle } =
            await setupBrowserWithExtension(webAppUrl);
        driver = d;
        extensionId = eId;
        extensionTitle = eTitle;
        webAppTitle = wTitle;
    }, 20000);

    afterAll(async () => {
        await driver.quit();
    });

    test("Create wallet", async () => {
        const debug = Debug("e2e-create");

        await switchToWindow(driver, extensionTitle);

        debug("Choosing password");

        let step1 = await getElementById(driver, "//button[@data-cy='data-cy-create-wallet-step-1']");
        await step1.click();

        let refreshMnemonicButton = await getElementById(
            driver,
            "//button[@data-cy='data-cy-create-wallet-generate-mnemonic']",
        );
        await refreshMnemonicButton.click();

        let mnemonicInput = await getElementById(driver, "//textarea[@data-cy='data-cy-create-wallet-mnemonic-input']");
        let mnemonic = mnemonicInput.getText();

        let checkBox = await getElementById(driver, "//label[@data-cy='data-cy-create-wallet-checkbox-input']");
        await checkBox.click();

        let step2 = await getElementById(driver, "//button[@data-cy='data-cy-create-wallet-step-2']");
        await step2.click();

        let mnemonicConfirmationInput = await getElementById(
            driver,
            "//textarea[@data-cy='data-cy-create-wallet-mnemonic-input-confirmation']",
        );
        await mnemonicConfirmationInput.sendKeys(mnemonic);

        let password = "foo";
        let passwordInput = await getElementById(driver, "//input[@data-cy='data-cy-create-wallet-password-input']");
        await passwordInput.sendKeys(password);

        debug("Creating wallet");
        let createWalletButton = await getElementById(
            driver,
            "//button[@data-cy='data-cy-create-wallet-button']",
        );
        await createWalletButton.click();

        debug("Getting wallet address");
        let addressField = await getElementById(driver, "//p[@data-cy='data-cy-wallet-address-text-field']");
        let address = await addressField.getText();

        let url = `${webAppUrl}/api/faucet/${address}`;
        debug("Calling faucet: %s", url);
        let response = await fetch(url, {
            method: "POST",
        });
        expect(response.ok).toBeTruthy();
        let body = await response.text();
        debug("Faucet response: %s", body);

        // TODO: Remove when automatic balance refreshing is implemented
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Waiting for balance update");
        let btcAmount = await getElementById(driver, "//p[@data-cy='data-cy-L-BTC-balance-text-field']", 20_000);
        debug("Found L-BTC amount: %s", await btcAmount.getText());

        let wallets = await driver.executeScript(
            "return window.localStorage.getItem('wallets')",
        );
        let pwd = await driver.executeScript(
            "return window.localStorage.getItem('wallets.demo.password')",
        );
        let xprv = await driver.executeScript(
            "return window.localStorage.getItem('wallets.demo.xprv')",
        );

        let setup_logger = Debug("setup");
        setup_logger(
            `await driver.executeScript("return window.localStorage.setItem('wallets.demo.wallets','${wallets}');",);`,
        );
        setup_logger(
            `await driver.executeScript("return window.localStorage.setItem('wallets.demo.password','${pwd}');",);`,
        );
        setup_logger(
            `await driver.executeScript("return window.localStorage.setItem('wallets.demo.xprv','${xprv}');",);`,
        );
        setup_logger(`Address: '${address}'`);
    }, 30_000);
});
