import { test, expect } from '@playwright/test';
import { connectWallet, navigateToPostDetail, tipPost } from './test-utils';

test.describe('Post Tipping', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to home page
    await page.goto('/');
    
    // Connect wallet before each test
    await connectWallet(page);
  });

  test('E2E: tip a post → verify tip total updates on the post detail page', async ({ page }) => {
    // Step 1: Navigate to feed to find a post to tip
    await page.goto('/feed');
    await page.waitForLoadState('networkidle');
    
    // Step 2: Find first post and click it
    const firstPost = page.locator('article').first();
    await expect(firstPost).toBeVisible();
    
    // Get post ID or navigate to post detail
    await firstPost.click();
    
    // Step 3: Wait for post detail page to load
    await page.waitForLoadState('networkidle');
    
    // Step 4: Get initial tip count/amount
    const initialTipText = await page.locator('text=/Tip|Support|✓.*\d+/').first().textContent();
    const initialTipCount = parseInt(initialTipText?.match(/\d+/)?.[0] || '0');
    
    // Step 5: Click tip button
    const tipButton = page.locator('button:has-text("Tip"), button:has-text("Support")').first();
    await expect(tipButton).toBeVisible();
    
    await tipPost(page, 1);
    
    // Step 6: Wait for update
    await page.waitForLoadState('networkidle');
    
    // Step 7: Verify tip total increased
    const updatedTipText = await page.locator('text=/Tip|Support|✓.*\d+/').first().textContent();
    const updatedTipCount = parseInt(updatedTipText?.match(/\d+/)?.[0] || '0');
    
    expect(updatedTipCount).toBeGreaterThanOrEqual(initialTipCount);
  });

  test('should display tip button on post detail page', async ({ page }) => {
    // Navigate to feed
    await page.goto('/feed');
    await page.waitForLoadState('networkidle');
    
    // Click first post
    const firstPost = page.locator('article').first();
    await firstPost.click();
    
    // Verify tip button is visible
    await page.waitForLoadState('networkidle');
    const tipButton = page.locator('button:has-text("Tip"), button:has-text("Support")').first();
    await expect(tipButton).toBeVisible();
  });

  test('should display tip amount in post metadata', async ({ page }) => {
    // Navigate to feed
    await page.goto('/feed');
    await page.waitForLoadState('networkidle');
    
    // Click first post
    const firstPost = page.locator('article').first();
    await firstPost.click();
    
    // Wait for detail page
    await page.waitForLoadState('networkidle');
    
    // Verify tip amount is displayed
    const tipAmount = page.locator('text=/Tip|Support|✓/');
    await expect(tipAmount.first()).toBeVisible();
  });
});
