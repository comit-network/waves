import * as assert from "assert";
import Debug from "debug";
import fetch from "node-fetch";
import { Builder, By, until, WebDriver } from "selenium-webdriver";
import { Driver } from "selenium-webdriver/firefox";

const firefox = require("selenium-webdriver/firefox");
const firefoxPath = require("geckodriver").path;

const getElementById = async (driver, xpath, timeout = 4000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

const setupBrowserWithExtension = async (webAppUrl: string) => {
    const service = new firefox.ServiceBuilder(firefoxPath);
    const options = new firefox.Options().headless();

    // it is of type firefox.Driver but typescript does not allow us to cast it
    // @ts-ignore
    let driver: Driver = new Builder()
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
    let extensionId = await extensionElement.getText();

    // load webapp again
    await driver.get(webAppUrl);
    let webAppTitle = await driver.getTitle();

    // Opens a new tab and switches to new tab
    await driver.switchTo().newWindow("tab");

    // Open extension
    let extensionUrl = `moz-extension://${extensionId}/popup.html`;
    await driver.get(`${extensionUrl}`);
    let extensionTitle = await driver.getTitle();
    return {
        driver,
        extensionId,
        extensionTitle,
        webAppTitle,
    };
};

// set testing wallet.
// seed words are: `bargain pretty shop spy travel toilet hero ridge critic race weapon elbow`
// wallet password is: `foo`.
// address is: `el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp`
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$fh5CD4WuA/JSKKnclw+Orw==$x3aZgNLWV8QzMPOffn+z7otM1/Up2yyrBgFLDkCNMoI=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','71dc4a79771da7a28e2ff1a805be3efd5fba436eb9280de0fe410297199e4975$7388ec81fed12d71b385216e48726f23dd863babbbe897af8486d08715776a5b12c8606704d4c00590899619d65cffe324293e8dc639deb0185db15f7f386db231453b78a9f7011af5ad75e113482506655ea2301b24ad0c14c6ddbe1224d2131bf647a74ed54b8befb76a0f0a90163d403a7a1a3976247302718be379619b');",
    );

    return "el1qqvq7q42zu99ky2g7n3hmh0yfr8eru0sxk6tutl3hlv240rd8rxyqrsukpsvsqdzc84dvmv6atmzp3f3hassdgmyx5cafy30dp";
};

const faucet = async (address: string) => {
    let faucetUrl = `http://localhost:3030/api/faucet/${address}`;
    let response = await fetch(faucetUrl, {
        method: "POST",
    });
    assert(response.ok);
};

const unlockWallet = async (driver: WebDriver, extensionTitle: string) => {
    let debug = Debug("e2e-unlock-wallet");

    await switchToWindow(driver, extensionTitle);

    await driver.navigate().refresh();

    debug("Unlocking wallet");
    let password = "foo";
    let passwordInput = await getElementById(driver, "//input[@data-cy='data-cy-unlock-wallet-password-input']");
    await passwordInput.sendKeys(password);

    let unlockWalletButton = await getElementById(driver, "//button[@data-cy='data-cy-unlock-wallet-button']");
    await unlockWalletButton.click();
};

async function getWindowHandle(driver: WebDriver, name: string) {
    let allWindowHandles = await driver.getAllWindowHandles();
    for (const windowHandle of allWindowHandles) {
        await driver.switchTo().window(windowHandle);
        const title = await driver.getTitle();
        if (title === name) {
            return windowHandle;
        }
    }
}

async function switchToWindow(driver: WebDriver, name: string) {
    await driver.switchTo().window(await getWindowHandle(driver, name));
}

export { faucet, getElementById, setupBrowserWithExtension, setupTestingWallet, switchToWindow, unlockWallet };
