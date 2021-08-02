import Debug from "debug";
import { until, WebDriver } from "selenium-webdriver";
import { getElementById, setupBrowserWithExtension, switchToWindow, unlockWallet } from "./utils";

describe("buy usdt/btc", () => {
    const webAppUrl = "http://localhost:3030";

    let driver;
    let extensionId: string;
    let webAppTitle: string;
    let extensionTitle: string;
    const debug = Debug("e2e-usdt-btc-buy");

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

    test("buy swap", async () => {
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
});

// set testing wallet.
// seed words are: `crucial pizza enhance learn banner family scrub save gas earn carpet bottom carbon cost scatter sign item dash magic balcony income cement option awkward`
// wallet password is: `foo`.
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$M3qALv1UBJhZdcduTXndIA==$Cn3O/TcSLYHyNlOrDAee7pPqKR0jgIXicUfSZPfcxrM=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','1073c5ea651a37d956b7062dbb711656655f9033df9094c2dc29b6d07ab8fa82$4529591215f662c22c5f99347110bfcf4e0b9221a31694c46d16ccc5260b288b1538a6a209d3410d048fac92d77f09eebd7a2e1db5fd4d6bcd49618fe755a1712c739822f63394ca38019225c51e40498fa49136a7e5161a51c3056e0092d7ba7681657e345be05c6da211a7d7442892f13237f7968f93d1b3f5883f0bfdf2');",
    );

    return "el1qqgupeqadsckv095dr7u90ukn3nsfs4gayfl3746kaf6l2nn0wrs8e5s4943wa0ven87ggpk9qgsdqz6779mgsj0783d9hau0d";
};
