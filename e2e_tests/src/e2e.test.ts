import * as assert from "assert";
import Debug from "debug";
import fetch from "node-fetch";
import { Builder, By, until } from "selenium-webdriver";

const firefox = require("selenium-webdriver/firefox");
const firefoxPath = require("geckodriver").path;

const getElementById = async (driver, xpath, timeout = 4000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

const getElementByClass = async (driver, className, timeout = 4000) => {
    const el = await driver.wait(until.elementLocated(By.className(className)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

describe("webdriver", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    beforeAll(async () => {
        const service = new firefox.ServiceBuilder(firefoxPath);
        const options = new firefox.Options().headless();

        driver = new Builder()
            .setFirefoxService(service)
            .forBrowser("firefox")
            .setFirefoxOptions(options)
            .build();

        await driver.installAddon("../extension/web-ext-artifacts/waves_wallet-0.0.1.zip", true);

        // this probably works forever unless we change something and then it won't work anymore
        await driver.get("about:debugging#/runtime/this-firefox");
        const extensionElement = await getElementById(
            driver,
            "//span[contains(text(),'waves_wallet')]//"
                + "parent::li/section/dl/div//dt[contains(text(),'Internal UUID')]/following-sibling::dd",
        );
        extensionId = await extensionElement.getText();

        // load webapp again
        await driver.get(webAppUrl);
        webAppTitle = await driver.getTitle();

        // Check we don't have other windows open already
        assert((await driver.getAllWindowHandles()).length === 1);

        // Opens a new tab and switches to new tab
        await driver.switchTo().newWindow("tab");

        // Open extension
        let extensionUrl = `moz-extension://${extensionId}/popup.html`;
        await driver.get(`${extensionUrl}`);
        extensionTitle = await driver.getTitle();
    }, 20000);

    afterAll(async () => {
        await driver.quit();
    });

    async function getWindowHandle(name: string) {
        let allWindowHandles = await driver.getAllWindowHandles();
        for (const windowHandle of allWindowHandles) {
            await driver.switchTo().window(windowHandle);
            const title = await driver.getTitle();
            if (title === name) {
                return windowHandle;
            }
        }
    }

    async function switchToWindow(name: string) {
        await driver.switchTo().window(await getWindowHandle(name));
    }

    test("Create wallet", async () => {
        const debug = Debug("e2e-create");

        await switchToWindow(extensionTitle);

        debug("Choosing password");
        let password = "foo";
        let passwordInput = await getElementByClass(driver, "data-cy-create-wallet-password-input");
        await passwordInput.sendKeys(password);

        debug("Creating wallet");
        let createWalletButton = await getElementByClass(driver, "data-cy-create-wallet-button");
        await createWalletButton.click();

        debug("Getting wallet address");
        let addressField = await getElementByClass(driver, "data-cy-wallet-address-text-field");
        let address = await addressField.getAttribute("value");
        debug(`Address found: ${address}`);

        let url = `${webAppUrl}/api/faucet/${address}`;
        debug("Calling faucet: %s", url);
        let response = await fetch(url, {
            method: "POST",
        });
        assert(response.ok);
        let body = await response.text();
        debug("Faucet response: %s", body);

        debug("Waiting for balance update");
        let btcAmount = await getElementByClass(driver, "data-cy-L-BTC-balance-text-field", 40_000);
        debug("Found L-BTC amount: %s", await btcAmount.getText());
    }, 30000);

    test("sell swap", async () => {
        const debug = Debug("e2e-sell");

        await switchToWindow(webAppTitle);
        await driver.navigate().refresh();
        await driver.sleep(2000);

        debug("Setting L-BTC amount");
        let alphaAmountInput = await getElementById(driver, "//div[@data-cy='Alpha-amount-input']//input");
        await alphaAmountInput.clear();
        await alphaAmountInput.sendKeys("0.4");

        debug("Clicking on swap button");
        let swapButton = await getElementById(driver, "//button[@data-cy='swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);

        debug("Signing and sending transaction");
        let signTransactionButton = await getElementByClass(driver, "data-cy-sign-tx-button", 20_000);
        await signTransactionButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        assert(url.includes("/swapped/"));
        debug("Swap successful");
    }, 40000);

    test("buy swap", async () => {
        const debug = Debug("e2e-buy");

        await switchToWindow(webAppTitle);
        await driver.get(webAppUrl);
        await driver.sleep(2000);

        debug("Switching assets");
        let switchAssetTypesButton = await getElementById(driver, "//button[@data-cy='exchange-asset-types-button']");
        await switchAssetTypesButton.click();

        debug("Setting L-USDt amount");
        let alphaAmountInput = await getElementById(driver, "//div[@data-cy='Alpha-amount-input']//input");
        await alphaAmountInput.clear();
        await alphaAmountInput.sendKeys("10000.0");

        debug("Clicking on swap button");
        let swapButton = await getElementById(driver, "//button[@data-cy='swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);

        debug("Signing and sending transaction");
        let signTransactionButton = await getElementByClass(driver, "data-cy-sign-tx-button", 20_000);
        await signTransactionButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        assert(url.includes("/swapped/"));
        debug("Swap successful");
    }, 40000);
});
