describe("LoopForge desktop smoke", () => {
  it("opens the home screen", async () => {
    const homePage = await $('[data-testid="home-page"]');
    await homePage.waitForDisplayed({ timeout: 20000 });
    await expect(browser).toHaveTitle("LoopForge");
    await expect(homePage).toBeDisplayed();
    await expect($('[data-testid="home-start-project-button"]')).toBeDisplayed();
  });
});
