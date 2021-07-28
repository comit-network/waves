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
// seed words are: `globe favorite camp draw action kid soul junk space soda genre vague name brisk female circle equal fix decade gloom elbow address genius noodle`
// wallet password is: `foo`.
// address is: `el1qqf09atu8jsmd25eclm8t5relx9q3x80yw3yncesd3ez2rgzy2pxkmamlcgrwlfa523rd4vjmcg4anyv2w89kk74k5p0af0pj5`
const setupTestingWallet = async (driver: WebDriver, extensionTitle: string) => {
    await switchToWindow(driver, extensionTitle);

    await driver.executeScript("return window.localStorage.setItem('wallets','demo');");
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.password','$rscrypt$0$DwgB$brb7JqhPEJbIxXOu/Jn3Aw==$8eWWdvAarl6IuOjViqAZuqhq05aD/YBjTAtioqtoC9U=$');",
    );
    await driver.executeScript(
        "return window.localStorage.setItem('wallets.demo.xprv','4b83ff6937ce2650354d02d9c7f6d9b828824c1dbe4d0795b3b14d9c9042dbca$0c130f39ada0ec0e9e0e2a2af91815f7e77f8ed64481467f19e42bcdfeaf88af2d10df9e9143223b71e053f67fad830ec102c7e977bd646cd1dec3a2718c97dc7e98cd349c7a3c2147f78c9b8b6436f68b96e2bf73f11ca496483175ad5dfa577d1efb40827455b05e3c51f9745ed8d4726d49f4a54470073efea876815f25');",
    );

    return "el1qqf09atu8jsmd25eclm8t5relx9q3x80yw3yncesd3ez2rgzy2pxkmamlcgrwlfa523rd4vjmcg4anyv2w89kk74k5p0af0pj5";
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
