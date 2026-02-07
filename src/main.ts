import './style.css';
import { AdDisplay } from './pages/AdDisplay';
import { ProductMenu } from './pages/ProductMenu';
import { categories, products } from './data/mockData';

type Page = 'ad' | 'menu' | 'password';

// ⚠️ 注意：此密码仅为防误触门禁，不构成任何安全鉴权。
// 页面部署在公开 GitHub Pages 上，任何人可查看源码获取此密码。
// 如需真正的访问控制，请参考 docs/ARCHITECTURE.md 10.2 节。
const ACCESS_PASSWORD = '594822';

// T014: 定时自动刷新（4 小时）
const AUTO_REFRESH_MS = 4 * 60 * 60 * 1000;

class App {
  private container: HTMLElement;
  private adDisplay: AdDisplay | null = null;
  private productMenu: ProductMenu | null = null;

  constructor() {
    this.container = document.getElementById('app')!;

    // T013: 全局异常兜底 — 未捕获异常 5 秒后自动刷新
    this.installGlobalErrorHandlers();

    // T014: 定时自动刷新
    setTimeout(() => location.reload(), AUTO_REFRESH_MS);

    // T011: localStorage 持久化（设备重启后免重复输入）
    if (localStorage.getItem('authenticated') === 'true') {
      this.navigateTo('ad');
    } else {
      this.showPasswordGate();
    }
  }

  // T013: 全局异常兜底
  private installGlobalErrorHandlers(): void {
    window.onerror = (_msg, _src, _line, _col, _err) => {
      console.error('[GlobalError]', _msg);
      this.scheduleRecovery();
    };
    window.onunhandledrejection = (ev) => {
      console.error('[UnhandledRejection]', ev.reason);
      this.scheduleRecovery();
    };
  }

  private recoveryScheduled = false;
  private scheduleRecovery(): void {
    if (this.recoveryScheduled) return;
    this.recoveryScheduled = true;
    console.warn('[Recovery] Scheduling page reload in 5s...');
    setTimeout(() => location.reload(), 5000);
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

    submit?.addEventListener('click', () => this.checkPassword(input, error));

    input?.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        this.checkPassword(input, error);
      }
    });

    input?.focus();
  }

  private checkPassword(input: HTMLInputElement, error: HTMLElement | null): void {
    const password = input.value;

    if (password === ACCESS_PASSWORD) {
      localStorage.setItem('authenticated', 'true');
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
    this.cleanup();

    switch (page) {
      case 'ad':
        // T012: 只有在有商品数据时才传入菜单跳转回调
        this.adDisplay = new AdDisplay(
          this.container,
          (categories.length > 0 && products.length > 0)
            ? () => this.navigateTo('menu')
            : null
        );
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
