import './style.css';
import { AdDisplay } from './pages/AdDisplay';
import { ProductMenu } from './pages/ProductMenu';

type Page = 'ad' | 'menu' | 'password';

// ⚠️ 注意：此密码仅为防误触门禁，不构成任何安全鉴权。
// 页面部署在公开 GitHub Pages 上，任何人可查看源码获取此密码。
// 如需真正的访问控制，请参考 docs/ARCHITECTURE.md 10.2 节。
const ACCESS_PASSWORD = '594822';

class App {
  private container: HTMLElement;
  private adDisplay: AdDisplay | null = null;
  private productMenu: ProductMenu | null = null;

  constructor() {
    this.container = document.getElementById('app')!;

    // Check if already authenticated in this session
    if (sessionStorage.getItem('authenticated') === 'true') {
      this.navigateTo('ad');
    } else {
      this.showPasswordGate();
    }
  }

  private showPasswordGate(): void {
    this.container.innerHTML = `
      <div class="password-gate">
        <div class="password-box">
          <div class="password-icon">🔐</div>
          <h2 class="password-title">绍兴黄酒专卖</h2>
          <p class="password-subtitle">请输入访问密码</p>
          <input type="password" id="password-input" class="password-input" placeholder="请输入密码" maxlength="10" />
          <button id="password-submit" class="password-submit">进入</button>
          <p id="password-error" class="password-error"></p>
        </div>
      </div>
    `;

    const input = document.getElementById('password-input') as HTMLInputElement;
    const submit = document.getElementById('password-submit');
    const error = document.getElementById('password-error');

    // Submit on button click
    submit?.addEventListener('click', () => this.checkPassword(input, error));

    // Submit on Enter key
    input?.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        this.checkPassword(input, error);
      }
    });

    // Auto focus
    input?.focus();
  }

  private checkPassword(input: HTMLInputElement, error: HTMLElement | null): void {
    const password = input.value;

    if (password === ACCESS_PASSWORD) {
      sessionStorage.setItem('authenticated', 'true');
      this.navigateTo('ad');
    } else {
      if (error) {
        error.textContent = '密码错误，请重试';
      }
      input.value = '';
      input.focus();
    }
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
