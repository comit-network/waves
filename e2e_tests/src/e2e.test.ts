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
        const options = new firefox.Options();

        driver = new Builder()
            .setFirefoxService(service)
            .forBrowser("firefox")
            .setFirefoxOptions(options)
            .build();

        await driver.get(webAppUrl);

        await driver.installAddon("../extension/zip/waves_wallet-0.0.1.zip", true);

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

        let passwordInput = await getElementById(driver, "//input[@data-cy='data-cy-create-wallet-password-input']");
        await passwordInput.sendKeys(password);

        debug("Creating wallet");
        let createWalletButton = await getElementById(
            driver,
            "//button[@data-cy='data-cy-create-or-unlock-wallet-button']",
        );
        await createWalletButton.click();

        debug("Getting wallet address");
        let addressField = await getElementById(driver, "//p[@data-cy='data-cy-wallet-address-text-field']");
        let address = await addressField.getText();
        debug(`Address found: ${address}`);

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

    test("sell swap", async () => {
        const debug = Debug("e2e-sell");

        await switchToWindow(webAppTitle);
        await driver.navigate().refresh();

        debug("Setting L-BTC amount");
        let btcAmountInput = await getElementById(driver, "//div[@data-cy='data-cy-L-BTC-amount-input']//input");
        await btcAmountInput.clear();
        await btcAmountInput.sendKeys("0.4");

        debug("Clicking on swap button");
        let swapButton = await getElementById(driver, "//button[@data-cy='data-cy-swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);

        // TODO: Remove when automatic pop-up refresh
        // happens based on signing state
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Signing and sending transaction");
        let signTransactionButton = await getElementById(driver, "//button[@data-cy='data-cy-sign-and-send-button']");
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

        debug("Switching assets");
        let switchAssetTypesButton = await getElementById(
            driver,
            "//button[@data-cy='data-cy-exchange-asset-types-button']",
        );
        await switchAssetTypesButton.click();

        debug("Setting L-USDt amount");
        let usdtAmountInput = await getElementById(driver, "//div[@data-cy='data-cy-USDt-amount-input']//input");
        await usdtAmountInput.clear();
        await usdtAmountInput.sendKeys("5000.0");

        debug("Clicking on swap button");
        let swapButton = await getElementById(driver, "//button[@data-cy='data-cy-swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(extensionTitle);

        // TODO: Remove when automatic pop-up refresh
        // happens based on signing state
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Signing and sending transaction");
        let signTransactionButton = await getElementById(
            driver,
            "//button[@data-cy='data-cy-sign-and-send-button']",
        );
        await signTransactionButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        assert(url.includes("/swapped/"));
        debug("Swap successful");
    }, 40000);

    test("borrow", async () => {
        const debug = Debug("e2e-borrow");

        debug("Navigating to borrow page");
        await switchToWindow(webAppTitle);
        await driver.get(`${webAppUrl}/borrow`);

        debug("Setting collateral amount");
        let principalAmountInput = await getElementById(
            driver,
            "//div[@data-cy='data-cy-principal-amount-input']//input",
        );
        await principalAmountInput.clear();

        // TODO: Careful changing this value until we fix a bug with the
        // computed collateral amount having too many decimal places
        await principalAmountInput.sendKeys("3000");

        debug("Clicking on take loan button");
        let takeLoanButton = await getElementById(driver, "//button[@data-cy='data-cy-take-loan-button']");
        await driver.wait(until.elementIsEnabled(takeLoanButton), 20000);
        await takeLoanButton.click();

        await switchToWindow(extensionTitle);

        // TODO: Remove when automatic pop-up refresh
        // happens based on signing state
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Signing loan");
        let signLoanButton = await getElementById(driver, "//button[@data-cy='data-cy-sign-loan-button']", 20_000);
        await signLoanButton.click();

        await switchToWindow(webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();

        // TODO: Change when we have dedicated success page for loans
        assert(url.includes("/swapped/"));
        debug("Loan successful");

        debug("Checking open loans");
        await driver.sleep(10000);
        await switchToWindow(extensionTitle);

        await driver.navigate().refresh();

        debug("Recording BTC balance before repayment");
        let btcBalanceBefore = await (await getElementById(driver, "//p[@data-cy='data-cy-L-BTC-balance-text-field']"))
            .getText();

        debug("Opening first open loan details");
        let openLoanButton = await getElementById(driver, "//button[@data-cy='data-cy-open-loan-0-button']");
        await openLoanButton.click();

        debug("Repaying first loan");
        let repayButton = await getElementById(driver, "//button[@data-cy='data-cy-repay-loan-0-button']");
        await repayButton.click();

        debug("Waiting for balance update");
        // TODO: Remove when automatic balance refreshing is
        // implemented
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        let btcBalanceAfter = await (await getElementById(driver, "//p[@data-cy='data-cy-L-BTC-balance-text-field']"))
            .getText();

        debug(`BTC balance before ${btcBalanceBefore}; BTC balance after ${btcBalanceAfter}`);
        assert((btcBalanceBefore < btcBalanceAfter));

        debug("Repayment successful");
    }, 40000);
});
