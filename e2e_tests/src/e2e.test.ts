import * as assert from "assert";
import { Builder, By, until } from "selenium-webdriver";
import { main } from "ts-node/dist/bin";

const firefox = require("selenium-webdriver/firefox");
const firefoxPath = require("geckodriver").path;

const getElementById = async (driver, xpath, timeout = 2000) => {
    const el = await driver.wait(until.elementLocated(By.xpath(xpath)), timeout);
    return await driver.wait(until.elementIsVisible(el), timeout);
};

describe("webdriver", () => {
    let driver;
    let extensionId;
    let websiteWindow;
    let extensionWindow;
    beforeAll(async () => {
        let service = new firefox.ServiceBuilder(firefoxPath);

        const options = new firefox.Options().headless();

        driver = new Builder()
            .setFirefoxService(service)
            .forBrowser("firefox")
            .setFirefoxOptions(options)
            .build();

        driver.getProfil;
        // this works :)
        await driver.installAddon("../extension/web-ext-artifacts/waves_wallet-0.0.1.zip", true);

        // this probably works forever unless we change something and then it won't work anymore
        await driver.get("about:debugging#/runtime/this-firefox");
        const extensionElement = await getElementById(
            driver,
            "//span[contains(text(),'waves_wallet')]//"
                + "parent::li/section/dl/div//dt[contains(text(),'Internal UUID')]/following-sibling::dd",
        );
        extensionId = await extensionElement.getText();

        // Store the ID of the original window
        websiteWindow = await driver.getWindowHandle();

        // Check we don't have other windows open already
        assert((await driver.getAllWindowHandles()).length === 1);

        // Open extension page in new tab
        let extensionUrl = `moz-extension://${extensionId}/popup.html?`;
        await driver.executeScript(`window.open("${extensionUrl}");`);

        // Wait for the new window or tab
        await driver.wait(
            async () => (await driver.getAllWindowHandles()).length === 2,
            10000,
        );

        const windows = await driver.getAllWindowHandles();
        extensionWindow = windows.find((w) => w != websiteWindow);

        await driver.switchTo().window(extensionWindow);

        // Assert that extension window is loaded
        await driver.wait(until.titleIs("Waves Wallet"), 10000);
    });

    afterAll(async () => {
        await driver.quit();
    });

    test("click swap", async () => {
        await driver.switchTo().window(websiteWindow);
        await driver.get("localhost:3004");
        await driver.wait(until.titleIs("Waves"), 10000);

        await getElementById(driver, "//button[@data-cy=\"swap-button\"]");
    });
});
