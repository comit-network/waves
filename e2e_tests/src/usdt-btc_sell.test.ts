import Debug from "debug";
import { until, WebDriver } from "selenium-webdriver";
import { getElementById, setupBrowserWithExtension, switchToWindow, unlockWallet } from "./utils";

describe("sell usdt/btc", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;

    const debug = Debug("e2e-usdt-btc-sell");

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

    test("sell swap", async () => {
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
});

// set testing wallet.
// seed words are: `strike sound divide secret upset crime search trick castle inhale trouble approve filter bonus good real token electric rebuild brand pupil amount path taste`
// wallet password is: `foo`.
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$QLDYJMgUIYi3/8vSRGiiNw==$N5F/psorzkxIhjNn+ZjmMHbUwy/dXB78v1m/TEe2O4Y=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','663d02eacee85af99d1a11672637f18fe1c8e6e545aab835735fe5b59c9f3786$677a6dee6e7f94539a5273ff560cd62b5553e641bd141a7a0d52f2731b4aa7f3aa10ba3661129ed2e32c1b3a3e5b9116f20bc347917bceb7bd07f93534071a0910f3f0b4814e9640ff80e16834c5893fbf3ce84427e455a9f319861a7b370f092bbc28776be7812056ee068dc733e6f952a0ae22489b6b71d34ba39570aa54');",
    );

    return "el1qqfahtc82zq03wcq6t2hwys5azxp5l5pggpsxq7f8h5jfpq0waced0tg6s9v0avhsam03ecyycdrgw4gzcsackynfpjmfd5l6d";
};
