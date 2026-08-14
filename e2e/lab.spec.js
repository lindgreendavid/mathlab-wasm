import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test("loads the WASM laboratory and changes scenarios", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Root-finding under the microscope" })).toBeVisible();
  await expect(page.getByText("Converged", { exact: true })).toBeVisible();

  await page.getByLabel("Function", { exact: true }).selectOption("newton-cycle");
  await page.getByLabel("Method", { exact: true }).selectOption("newton");
  await page.getByLabel("First value", { exact: true }).fill("0");
  await page.getByRole("button", { name: "Run the method" }).click();
  await expect(page.getByText("Cycle detected", { exact: true })).toBeVisible();
});

test("shows safeguarded interpolation and bisection from the Rust trace", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Converged", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: /SAFEGUARD ACTIVE/ }).click();
  await expect(page.getByLabel("Method", { exact: true })).toHaveValue("safeguarded");
  await expect(page.getByLabel("Function", { exact: true })).toHaveValue("skewed");
  await expect(page.getByText("inverse quadratic", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("bisection", { exact: true }).first()).toBeVisible();
});

test("loads the frozen conditioning report and switches cases", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("small residual high sensitivity", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: /Repeated root/ }).click();
  await expect(page.getByText("simple root diagnostic unavailable", { exact: true })).toBeVisible();
  await expect(page.getByText("Unavailable", { exact: true })).toHaveCount(2);
  await expect(page.getByText(/derivative vanishes at the multiple root/i)).toBeVisible();
});

test("has no automatically detectable WCAG A/AA violations", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Converged", { exact: true })).toBeVisible();
  const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]).analyze();
  expect(results.violations).toEqual([]);
});

test("fits a narrow mobile viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(page.getByText("Converged", { exact: true })).toBeVisible();
  const dimensions = await page.evaluate(() => ({ viewport: innerWidth, document: document.documentElement.scrollWidth }));
  expect(dimensions.document).toBeLessThanOrEqual(dimensions.viewport);
});
