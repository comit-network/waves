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

describe("e2e tests", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    beforeAll(async () => {
        const debug = Debug("e2e-setup");
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
        // set testing wallet.
        // seed words are: `bargain pretty shop spy travel toilet hero ridge critic race weapon elbow`
        // wallet password is: `foo`.
        // address is: `el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp`
        await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
        await driver.executeScript(
            "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$fh5CD4WuA/JSKKnclw+Orw==$x3aZgNLWV8QzMPOffn+z7otM1/Up2yyrBgFLDkCNMoI=$');",
        );
        await driver.executeScript(
            "return window.localStorage.setItem('wallets.demo.xprv','71dc4a79771da7a28e2ff1a805be3efd5fba436eb9280de0fe410297199e4975$7388ec81fed12d71b385216e48726f23dd863babbbe897af8486d08715776a5b12c8606704d4c00590899619d65cffe324293e8dc639deb0185db15f7f386db231453b78a9f7011af5ad75e113482506655ea2301b24ad0c14c6ddbe1224d2131bf647a74ed54b8befb76a0f0a90163d403a7a1a3976247302718be379619b');",
        );

        let faucetUrl =
            `http://localhost:3030/api/faucet/el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp`;
        let response = await fetch(faucetUrl, {
            method: "POST",
        });
        assert(response.ok);

        await driver.navigate().refresh();

        // unlock wallet
        debug("Unlocking wallet");
        let password = "foo";
        let passwordInput = await getElementById(driver, "//input[@data-cy='data-cy-unlock-wallet-password-input']");
        await passwordInput.sendKeys(password);

        let unlockWalletButton = await getElementById(driver, "//button[@data-cy='data-cy-unlock-wallet-button']");
        await unlockWalletButton.click();

        debug("Getting wallet address");
        let addressField = await getElementById(driver, "//p[@data-cy='data-cy-wallet-address-text-field']");
        let address = await addressField.getText();
        debug(`Address found: ${address}`);
        assert(
            "el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp"
                === address,
        );
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
