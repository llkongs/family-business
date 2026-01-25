import { storeInfo, mediaPlaylist } from '../data/mockData';

export class AdDisplay {
  private container: HTMLElement;
  private carouselIndex: number = 0;
  private carouselTimer: number | null = null;
  private onEnterMenu: () => void;

  constructor(container: HTMLElement, onEnterMenu: () => void) {
    this.container = container;
    this.onEnterMenu = onEnterMenu;
    this.render();
    this.startCarousel();
  }

  private render(): void {
    const images = mediaPlaylist.filter(m => m.type === 'image');

    this.container.innerHTML = `
      <div class="ad-display">
        <!-- Header Bar -->
        <div class="header-bar">
          <div class="store-info">
            <span class="store-name">🏺 ${storeInfo.name}</span>
            <span class="store-phone">${storeInfo.phone}</span>
          </div>
          <div class="qr-code">
            <img src="${storeInfo.qrCodeUrl}" alt="QR Code" />
          </div>
        </div>

        <!-- Main Carousel Section - Full Height -->
        <div class="main-carousel" id="main-carousel">
          <div class="carousel-container">
            <div class="carousel-track" id="carousel-track">
              ${images.map((img, i) => `
                <div class="carousel-slide ${i === 0 ? 'active' : ''}" data-index="${i}">
                  <img src="${img.url}" alt="${img.title || ''}" />
                  <div class="slide-title">${img.title || ''}</div>
                </div>
              `).join('')}
            </div>
          </div>
          
          <!-- Navigation Dots -->
          <div class="carousel-dots" id="carousel-dots">
            ${images.map((_, i) => `
              <div class="carousel-dot ${i === 0 ? 'active' : ''}" data-index="${i}"></div>
            `).join('')}
          </div>
          
          <!-- Enter Menu Button -->
          <button class="enter-menu-btn" id="enter-menu-btn">
            🍶 点击查看产品
          </button>
          
          <!-- Brand Indicators -->
          <div class="brand-indicators">
            <span class="brand-tag active" data-brand="guyuelongshan">古越龙山</span>
            <span class="brand-tag" data-brand="kuaijishan">会稽山</span>
            <span class="brand-tag" data-brand="nverhong">女儿红</span>
          </div>
        </div>
      </div>
    `;

    this.attachEventListeners();
  }

  private attachEventListeners(): void {
    const enterMenuBtn = document.getElementById('enter-menu-btn');
    const mainCarousel = document.getElementById('main-carousel');
    const dots = document.querySelectorAll('.carousel-dot');

    // Enter menu button
    enterMenuBtn?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.cleanup();
      this.onEnterMenu();
    });

    // Click on carousel area (except button)
    mainCarousel?.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      if (target.id !== 'enter-menu-btn' && !target.classList.contains('carousel-dot')) {
        this.cleanup();
        this.onEnterMenu();
      }
    });

    // Dots click
    dots.forEach(dot => {
      dot.addEventListener('click', (e) => {
        e.stopPropagation();
        const index = parseInt((dot as HTMLElement).dataset.index || '0');
        this.goToSlide(index);
      });
    });

    // Touch swipe support
    let startX = 0;
    mainCarousel?.addEventListener('touchstart', (e) => {
      startX = e.touches[0].clientX;
    });

    mainCarousel?.addEventListener('touchend', (e) => {
      const endX = e.changedTouches[0].clientX;
      const diff = startX - endX;

      if (Math.abs(diff) > 50) {
        if (diff > 0) {
          this.nextSlide();
        } else {
          this.prevSlide();
        }
      }
    });
  }

  private startCarousel(): void {
    const images = mediaPlaylist.filter(m => m.type === 'image');
    const interval = images[0]?.duration || 5000;

    this.carouselTimer = window.setInterval(() => {
      this.nextSlide();
    }, interval);
  }

  private nextSlide(): void {
    const images = mediaPlaylist.filter(m => m.type === 'image');
    this.carouselIndex = (this.carouselIndex + 1) % images.length;
    this.updateCarouselPosition();
  }

  private prevSlide(): void {
    const images = mediaPlaylist.filter(m => m.type === 'image');
    this.carouselIndex = (this.carouselIndex - 1 + images.length) % images.length;
    this.updateCarouselPosition();
  }

  private goToSlide(index: number): void {
    this.carouselIndex = index;
    this.updateCarouselPosition();

    // Reset timer
    if (this.carouselTimer) {
      clearInterval(this.carouselTimer);
      this.startCarousel();
    }
  }

  private updateCarouselPosition(): void {
    const track = document.getElementById('carousel-track');
    const dots = document.querySelectorAll('.carousel-dot');
    const slides = document.querySelectorAll('.carousel-slide');
    const brandTags = document.querySelectorAll('.brand-tag');

    if (track) {
      track.style.transform = `translateX(-${this.carouselIndex * 100}%)`;
    }

    // Update active slide for Ken Burns effect
    slides.forEach((slide, i) => {
      slide.classList.toggle('active', i === this.carouselIndex);
    });

    // Update dots
    dots.forEach((dot, i) => {
      dot.classList.toggle('active', i === this.carouselIndex);
    });

    // Update brand indicators (0-1: 古越龙山, 2-3: 会稽山, 4-5: 女儿红)
    const brandIndex = Math.floor(this.carouselIndex / 2);
    brandTags.forEach((tag, i) => {
      tag.classList.toggle('active', i === brandIndex);
    });
  }

  public cleanup(): void {
    if (this.carouselTimer) {
      clearInterval(this.carouselTimer);
      this.carouselTimer = null;
    }
  }

  public destroy(): void {
    this.cleanup();
    this.container.innerHTML = '';
  }
}
