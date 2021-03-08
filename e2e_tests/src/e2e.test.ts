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

describe("webdriver", () => {
    let driver;
    let extensionId;
    // TODO: Do not hard-code website URL
    const webAppUrl = "http://localhost:3030";
    const webAppTitle = "Waves";
    const extensionTitle = "Waves Wallet";

    beforeAll(async () => {
        const debug = Debug("e2e-before");
        let service = new firefox.ServiceBuilder(firefoxPath);

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
        // Assert that webapp window is loaded
        await driver.wait(until.titleIs(webAppTitle), 10000);

        // Check we don't have other windows open already
        assert((await driver.getAllWindowHandles()).length === 1);

        // Opens a new tab and switches to new tab
        await driver.switchTo().newWindow("tab");

        // Open extension
        let extensionUrl = `moz-extension://${extensionId}/popup.html`;
        await driver.get(`${extensionUrl}`);

        // Assert that extension window is loaded
        await driver.wait(until.titleIs(extensionTitle), 10000);
    }, 20000);

    afterAll(async () => {
        // await driver.quit();
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
        debug("Testing creating a wallet");
        await switchToWindow(extensionTitle);
        debug("Switched to extension");

        let password = "foo";
        debug("Setting password");
        let passwordInput = await getElementById(driver, "//input[@data-cy='create-wallet-password-input']");
        await passwordInput.sendKeys(password);

        debug("Creating wallet");
        let createWalletButton = await getElementById(driver, "//button[@data-cy='create-wallet-button']");
        await createWalletButton.click();

        debug("Getting address");
        let addressField = await getElementById(driver, "//p[@data-cy='wallet-address-text-field']");
        let address = await addressField.getText();
        debug(`Address found: ${address}`);

        debug("Calling faucet");
        await fetch(`${webAppUrl}/api/faucet/${address}`, {
            method: "POST",
        });

        debug("Waiting for balance update");
        await driver.wait(
            async () => {
                let round = 0;
                let max = 10;
                while (round++ < max) {
                    debug("Retry %s/%s", round, max);
                    try {
                        await driver.navigate().refresh();
                        let btcAmount = await getElementById(driver, "//p[@data-cy='L-BTC-balance-text-field']");
                        debug("Found btc amount: %s", await btcAmount.getText());
                        return true;
                    } catch (_) {
                        // ignore
                    }
                }
            },
            20000,
        );
    }, 30000);

    test("sell swap", async () => {
        const debug = Debug("e2e-sell");
        debug("Testing sell swap");

        await switchToWindow(webAppTitle);
        await driver.navigate().refresh();
        await driver.sleep(2000);
        debug("Setting alpha input amount");
        let alphaAmountInput = await getElementById(driver, "//div[@data-cy='Alpha-amount-input']//input");
        await alphaAmountInput.clear();
        await alphaAmountInput.sendKeys("0.4");

        debug("Checking if button is available.");

        let swapButton = await getElementById(driver, "//button[@data-cy='swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);
        await driver.sleep(5000);
        await driver.navigate().refresh();
        await driver.sleep(1000);

        debug("Signing transaction");
        let signTransactionButton = await getElementById(driver, "//button[@data-cy='sign-tx-button']");
        await signTransactionButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        assert(url.includes("/swapped/"));
    }, 40000);

    test("buy swap", async () => {
        const debug = Debug("e2e-buy");
        debug("Testing buy swap");

        await switchToWindow(webAppTitle);
        await driver.get(webAppUrl);
        await driver.sleep(2000);
        debug("Switching assets");
        let switchAssetTypesButton = await getElementById(driver, "//button[@data-cy='exchange-asset-types-button']");
        await switchAssetTypesButton.click();

        debug("Setting alpha input amount");
        let alphaAmountInput = await getElementById(driver, "//div[@data-cy='Alpha-amount-input']//input");
        await alphaAmountInput.clear();
        await alphaAmountInput.sendKeys("10000.0");

        debug("Checking if button is available.");

        let swapButton = await getElementById(driver, "//button[@data-cy='swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);
        await driver.sleep(5000);
        await driver.navigate().refresh();
        await driver.sleep(1000);

        debug("Signing transaction");
        let signTransactionButton = await getElementById(driver, "//button[@data-cy='sign-tx-button']");
        await signTransactionButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        assert(url.includes("/swapped/"));
    }, 40000);
});
