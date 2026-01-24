import './style.css';
import { AdDisplay } from './pages/AdDisplay';
import { ProductMenu } from './pages/ProductMenu';

type Page = 'ad' | 'menu';

class App {
  private container: HTMLElement;
  private adDisplay: AdDisplay | null = null;
  private productMenu: ProductMenu | null = null;

  constructor() {
    this.container = document.getElementById('app')!;
    this.navigateTo('ad');
  }

  private navigateTo(page: Page): void {
    // Cleanup current page
    this.cleanup();

    switch (page) {
      case 'ad':
        this.adDisplay = new AdDisplay(this.container, () => this.navigateTo('menu'));
        break;
      case 'menu':
        this.productMenu = new ProductMenu(this.container, () => this.navigateTo('ad'));
        break;
    }
  }

  private cleanup(): void {
    if (this.adDisplay) {
      this.adDisplay.destroy();
      this.adDisplay = null;
    }
    if (this.productMenu) {
      this.productMenu.destroy();
      this.productMenu = null;
    }
  }
}

// Initialize app
new App();
