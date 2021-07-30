import Debug from "debug";
import { promises as fsp } from "fs";
import fetch from "node-fetch";
import { Builder, By, until, WebDriver } from "selenium-webdriver";
import { Driver } from "selenium-webdriver/firefox";

const firefox = require("selenium-webdriver/firefox");
const firefoxPath = require("geckodriver").path;

const getElementById = async (driver, xpath, timeout = 4000) => {
    try {
        const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
        return await driver.wait(until.elementIsVisible(el), timeout);
    } catch (e) {
        const filename = xpath.replace(/[^\w\s]/gi, "");
        await takeScreenshot(driver, `./screenshots/error-${filename}.png`);
        throw e;
    }
};

const setupBrowserWithExtension = async (webAppUrl: string) => {
    const service = new firefox.ServiceBuilder(firefoxPath);
    let options = new firefox.Options();
    if (!process.env.BROWSER) {
        options = options.headless();
    }

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

async function takeScreenshot(driver, file) {
    let image = await driver.takeScreenshot();
    await fsp.writeFile(file, image, "base64");
}

export { getElementById, setupBrowserWithExtension, switchToWindow, unlockWallet };
