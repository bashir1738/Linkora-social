import { test, expect } from '@playwright/test';
import { connectWallet, createPost, waitForPostInFeed, clickPostInFeed, navigateToFeed } from './test-utils';

test.describe('Post Creation & Feed', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to home page
    await page.goto('/');
    
    // Connect wallet before each test
    await connectWallet(page);
  });

  test('E2E: create post → verify post appears on the feed', async ({ page }) => {
    // Step 1: Navigate to feed or create post
    await navigateToFeed(page);
    
    // Step 2: Create a new post
    const postContent = `Test post ${Date.now()}`;
    await createPost(page, postContent);
    
    // Step 3: Verify post appears in feed
    await waitForPostInFeed(page, postContent);
    
    // Step 4: Verify the post is visible with correct content
    const postElement = page.locator(`text="${postContent}"`).first();
    await expect(postElement).toBeVisible();
  });

  test('should display feed with posts', async ({ page }) => {
    // Navigate to feed
    await navigateToFeed(page);
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Verify feed page is displayed
    const feedContainer = page.locator('[data-testid="feed"], article').first();
    await expect(feedContainer).toBeVisible();
  });

  test('should open post detail when clicking on a post', async ({ page }) => {
    // Navigate to feed
    await navigateToFeed(page);
    
    // Wait for posts to load
    await page.waitForLoadState('networkidle');
    
    // Find first post
    const firstPost = page.locator('article').first();
    await expect(firstPost).toBeVisible();
    
    // Click on the post
    await firstPost.click();
    
    // Verify we navigated to post detail page
    await page.waitForLoadState('networkidle');
    const postDetailHeading = page.locator('h1, h2, [data-testid="post-detail"]').first();
    await expect(postDetailHeading).toBeVisible();
  });

  test('should display post metadata (author, timestamp)', async ({ page }) => {
    // Navigate to feed
    await navigateToFeed(page);
    
    // Wait for posts to load
    await page.waitForLoadState('networkidle');
    
    // Check first post has metadata
    const firstPost = page.locator('article').first();
    const authorElement = firstPost.locator('text=/[GS][A-Z0-9]{4}.*[A-Z0-9]{4}|@[\w]+/');
    const timestampElement = firstPost.locator('text=/ago|Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec/');
    
    await expect(authorElement).toBeVisible();
    await expect(timestampElement).toBeVisible();
  });
});
