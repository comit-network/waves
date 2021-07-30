import Debug from "debug";
import { until, WebDriver } from "selenium-webdriver";
import { getElementById, setupBrowserWithExtension, switchToWindow, unlockWallet } from "./utils";

describe("borrow test", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;
    const debug = Debug("e2e-borrow");

    beforeAll(async () => {
        let { driver: d, extensionId: eId, extensionTitle: eTitle, webAppTitle: wTitle } =
            await setupBrowserWithExtension(webAppUrl);
        driver = d;
        extensionId = eId;
        extensionTitle = eTitle;
        webAppTitle = wTitle;

        debug("Setting up testing wallet");
        let address = await setupTestingWallet(driver, extensionTitle);

        debug("Unlocking wallet");
        await unlockWallet(driver, extensionTitle);
    }, 20000);

    afterAll(async () => {
        await driver.quit();
    });

    test("borrow", async () => {
        debug("Navigating to borrow page");
        await switchToWindow(driver, webAppTitle);
        await driver.get(`${webAppUrl}/borrow`);

        debug("Setting collateral amount");
        let principalAmountInput = await getElementById(
            driver,
            "//div[@data-cy='data-cy-principal-amount-input']//input",
        );
        await principalAmountInput.clear();

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

// set testing wallet.
// seed words are: `town exhaust stamp palace grab crater jacket industry day dwarf various jacket frozen cricket issue bubble advance zebra raven ivory label capital strong uphold`
// wallet password is: `foo`.
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$7BzKF9q5Hg/nILiT0M7X+Q==$xh7ImFcmYCrBdqsEj2jBaOZHPyg0hnWCkIUICcLmJuQ=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','733889793b645460290a16b9d19166b761fb849154e5485848c6fccc6b760442$48c3859aae6a8480447065703096e5dfde215dbf3e80306a05ceaa262d1b9b52881390a22bc4411a2a885e704f45bede9c0ec74866292c9ad86a8156ff4d16dfcdb72b8e8c481392fd32bc181b0b4ff376651ba6474feb60a2f05a638671037e4efaf680209deed0ae183fba65bb437725a9509ae9ee07f06a1ee3312c33c7');",
    );

    return "el1qq0zel5lg55nvhv9kkrq8gme8hnvp0lemuzcmu086dn2m8laxjgkewkhqnh8vxdnlp4cejs3925j0gu9n9krdgmqm89vku0kc8";
};
