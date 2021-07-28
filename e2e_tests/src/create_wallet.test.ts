import * as assert from "assert";
import Debug from "debug";
import fetch from "node-fetch";
import { By, until } from "selenium-webdriver";
import { setupBrowserWithExtension, switchToWindow } from "./utils";

const getElementById = async (driver, xpath, timeout = 4000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

describe("webdriver", () => {
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

        let mnemonic =
            "globe favorite camp draw action kid soul junk space soda genre vague name brisk female circle equal fix decade gloom elbow address genius noodle";

        let mnemonicInput = await getElementById(driver, "//textarea[@data-cy='data-cy-create-wallet-mnemonic-input']");
        await mnemonicInput.sendKeys(mnemonic);

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
        debug(`Address found: ${address}`);

        // TODO: re-enable faucet again
        let url = `${webAppUrl}/api/faucet/${address}`;
        debug("Calling faucet: %s", url);
        let response = await fetch(url, {
            method: "POST",
        });
        assert(response.ok);
        let body = await response.text();
        debug("Faucet response: %s", body);

        // TODO: Remove when automatic balance refreshing is
        // implemented
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Waiting for balance update");
        let btcAmount = await getElementById(driver, "//p[@data-cy='data-cy-L-BTC-balance-text-field']", 20_000);
        debug("Found L-BTC amount: %s", await btcAmount.getText());
    }, 30_000);
});
