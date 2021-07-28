import Debug from "debug";
import { until } from "selenium-webdriver";
import {
    faucet,
    getElementById,
    setupBrowserWithExtension,
    setupTestingWallet,
    switchToWindow,
    unlockWallet,
} from "./utils";

describe("e2e tests", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    beforeAll(async () => {
        const debug = Debug("e2e-setup");

        let { driver: d, extensionId: eId, extensionTitle: eTitle, webAppTitle: wTitle } =
            await setupBrowserWithExtension(webAppUrl);
        driver = d;
        extensionId = eId;
        extensionTitle = eTitle;
        webAppTitle = wTitle;

        debug("Setting up testing wallet");
        let address = await setupTestingWallet(driver, extensionTitle);

        debug("Funding testing wallet");
        await faucet(address);

        debug("Unlocking wallet");
        await unlockWallet(driver, extensionTitle);
    }, 20000);

    afterAll(async () => {
        await driver.quit();
    });

    test("sell swap", async () => {
        const debug = Debug("e2e-sell");

        await switchToWindow(driver, webAppTitle);
        await driver.navigate().refresh();

        debug("Setting L-BTC amount");
        let btcAmountInput = await getElementById(driver, "//div[@data-cy='data-cy-L-BTC-amount-input']//input");
        await btcAmountInput.clear();
        await btcAmountInput.sendKeys("0.4");

        debug("Clicking on swap button");
        let swapButton = await getElementById(driver, "//button[@data-cy='data-cy-swap-button']");
        await driver.wait(until.elementIsEnabled(swapButton), 20000);
        await swapButton.click();

        await switchToWindow(driver, extensionTitle);

        // TODO: Remove when automatic pop-up refresh
        // happens based on signing state
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Signing and sending transaction");
        let signTransactionButton = await getElementById(driver, "//button[@data-cy='data-cy-sign-and-send-button']");
        await signTransactionButton.click();

        await switchToWindow(driver, webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        expect(url).toContain("/swapped/");
        debug("Swap successful");
    }, 40000);

    test("buy swap", async () => {
        const debug = Debug("e2e-buy");

        await switchToWindow(driver, webAppTitle);
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

        await switchToWindow(driver, extensionTitle);

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

        await switchToWindow(driver, webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();
        expect(url).toContain("/swapped/");
        debug("Swap successful");
    }, 40000);

    test("borrow", async () => {
        const debug = Debug("e2e-borrow");

        debug("Navigating to borrow page");
        await switchToWindow(driver, webAppTitle);
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

        await switchToWindow(driver, extensionTitle);

        // TODO: Remove when automatic pop-up refresh
        // happens based on signing state
        await new Promise(r => setTimeout(r, 10_000));
        await driver.navigate().refresh();

        debug("Signing loan");
        let signLoanButton = await getElementById(driver, "//button[@data-cy='data-cy-sign-loan-button']", 20_000);
        await signLoanButton.click();

        await switchToWindow(driver, webAppTitle);

        await driver.sleep(2000);
        let url = await driver.getCurrentUrl();

        // TODO: Change when we have dedicated success page for loans
        expect(url).toContain("/swapped/");
        debug("Loan successful");

        debug("Checking open loans");
        await driver.sleep(10000);
        await switchToWindow(driver, extensionTitle);

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
        expect(Number(btcBalanceBefore)).toBeLessThan(Number(btcBalanceAfter));

        debug("Repayment successful");
    }, 40000);
});
