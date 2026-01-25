import { categories, products, getProductsByCategory } from '../data/mockData';
import type { Product } from '../data/mockData';

export class ProductMenu {
  private container: HTMLElement;
  private onBack: () => void;
  private activeCategory: string = 'hot';

  constructor(container: HTMLElement, onBack: () => void) {
    this.container = container;
    this.onBack = onBack;
    this.render();
  }

  private render(): void {
    this.container.innerHTML = `
      <div class="product-menu">
        <!-- Header -->
        <div class="menu-header">
          <button class="back-btn" id="back-btn">←</button>
          <span class="menu-title">菜单</span>
          <div style="width: 40px;"></div>
        </div>

        <!-- Content -->
        <div class="menu-content">
          <!-- Category Navigation -->
          <nav class="category-nav" id="category-nav">
            ${categories.map(cat => `
              <div class="category-item ${cat.id === this.activeCategory ? 'active' : ''}" 
                   data-category="${cat.id}">
                ${cat.icon || ''} ${cat.name}
              </div>
            `).join('')}
          </nav>

          <!-- Product List -->
          <div class="product-list" id="product-list">
            ${this.renderProductSections()}
          </div>
        </div>
      </div>
    `;

    this.attachEventListeners();
  }

  private renderProductSections(): string {
    return categories.map(cat => {
      const categoryProducts = getProductsByCategory(cat.id);
      if (categoryProducts.length === 0) return '';

      return `
        <section class="category-section" data-section="${cat.id}">
          <h2 class="category-title">${cat.icon || ''} ${cat.name}</h2>
          ${categoryProducts.map(product => this.renderProductCard(product)).join('')}
        </section>
      `;
    }).join('');
  }

  private renderProductCard(product: Product): string {
    return `
      <div class="product-card" data-product-id="${product.id}">
        <img class="product-image" src="${product.image}" alt="${product.name}" 
             onerror="this.src='https://via.placeholder.com/80?text=图片'" />
        <div class="product-info">
          <div>
            <h3 class="product-name">${product.name}</h3>
            <p class="product-desc">${product.description}</p>
          </div>
          <div class="product-price">仅供展示</div>
        </div>
      </div>
    `;
  }

  private attachEventListeners(): void {
    // Back button
    const backBtn = document.getElementById('back-btn');
    backBtn?.addEventListener('click', () => {
      this.onBack();
    });

    // Category navigation
    const categoryNav = document.getElementById('category-nav');
    categoryNav?.addEventListener('click', (e) => {
      const target = (e.target as HTMLElement).closest('.category-item');
      if (target) {
        const categoryId = (target as HTMLElement).dataset.category;
        if (categoryId) {
          this.scrollToCategory(categoryId);
        }
      }
    });

    // Product cards
    const productCards = document.querySelectorAll('.product-card');
    productCards.forEach(card => {
      card.addEventListener('click', () => {
        const productId = (card as HTMLElement).dataset.productId;
        if (productId) {
          const product = products.find(p => p.id === productId);
          if (product) {
            this.showProductDetail(product);
          }
        }
      });
    });

    // Product list scroll - update active category
    const productList = document.getElementById('product-list');
    productList?.addEventListener('scroll', () => {
      this.updateActiveCategoryOnScroll();
    });
  }

  private scrollToCategory(categoryId: string): void {
    const section = document.querySelector(`[data-section="${categoryId}"]`);
    if (section) {
      section.scrollIntoView({ behavior: 'smooth', block: 'start' });
      this.setActiveCategory(categoryId);
    }
  }

  private setActiveCategory(categoryId: string): void {
    this.activeCategory = categoryId;
    const items = document.querySelectorAll('.category-item');
    items.forEach(item => {
      item.classList.toggle('active', (item as HTMLElement).dataset.category === categoryId);
    });
  }

  private updateActiveCategoryOnScroll(): void {
    const productList = document.getElementById('product-list');
    if (!productList) return;

    const sections = document.querySelectorAll('.category-section');
    const scrollTop = productList.scrollTop;

    for (const section of sections) {
      const sectionEl = section as HTMLElement;
      const offsetTop = sectionEl.offsetTop - productList.offsetTop;
      const height = sectionEl.offsetHeight;

      if (scrollTop >= offsetTop - 50 && scrollTop < offsetTop + height) {
        const categoryId = sectionEl.dataset.section;
        if (categoryId && categoryId !== this.activeCategory) {
          this.setActiveCategory(categoryId);
        }
        break;
      }
    }
  }

  private showProductDetail(product: Product): void {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.id = 'product-modal';
    modal.innerHTML = `
      <div class="modal-content">
        <button class="modal-close" id="modal-close">×</button>
        <img class="modal-image" src="${product.image}" alt="${product.name}"
             onerror="this.src='https://via.placeholder.com/400x250?text=图片'" />
        <div class="modal-body">
          <h2 class="modal-title">${product.name}</h2>
          <p class="modal-description">${product.description}</p>
          <div class="modal-price">仅供展示</div>
        </div>
      </div>
    `;

    document.body.appendChild(modal);

    // Close button
    const closeBtn = document.getElementById('modal-close');
    closeBtn?.addEventListener('click', () => this.closeProductDetail());

    // Click overlay to close
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        this.closeProductDetail();
      }
    });
  }

  private closeProductDetail(): void {
    const modal = document.getElementById('product-modal');
    modal?.remove();
  }

  public destroy(): void {
    this.closeProductDetail();
    this.container.innerHTML = '';
  }
}
