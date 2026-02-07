import Hls from 'hls.js';
import { storeInfo, mediaPlaylist } from '../data/mockData';

export class AdDisplay {
  private container: HTMLElement;
  private onEnterMenu: () => void;
  private hls: Hls | null = null;
  private videoEl: HTMLVideoElement | null = null;
  private videos = mediaPlaylist.filter(m => m.type === 'video');
  private images = mediaPlaylist.filter(m => m.type === 'image');
  private currentVideoIndex = 0;
  private carouselIndex = 0;
  private carouselTimer: number | null = null;

  constructor(container: HTMLElement, onEnterMenu: () => void) {
    this.container = container;
    this.onEnterMenu = onEnterMenu;
    this.render();
    this.initVideo();
    this.startCarousel();
  }

  private render(): void {
    const hasImages = this.images.length > 0;

    this.container.innerHTML = `
      <div class="ad-display">
        <!-- Header Bar -->
        <div class="header-bar">
          <div class="store-info">
            <span class="store-name">\u{1F3FA} ${storeInfo.name}</span>
            <span class="store-phone">${storeInfo.phone}</span>
          </div>
          <div class="header-right">
            <div class="qr-code">
              <img src="${storeInfo.qrCodeUrl}" alt="QR Code" />
            </div>
          </div>
        </div>

        <!-- Video Section -->
        <div class="video-section">
          <video id="ad-video" playsinline></video>
          <div class="video-title" id="video-title"></div>
        </div>

        <!-- Image Carousel -->
        <div class="image-carousel">
          ${hasImages ? `
            <div class="carousel-container">
              <div class="carousel-track" id="carousel-track">
                ${this.images.map((img, i) => `
                  <div class="carousel-slide ${i === 0 ? 'active' : ''}" data-index="${i}">
                    <img src="${img.url}" alt="${img.title || ''}" />
                    ${img.title ? `<div class="slide-title">${img.title}</div>` : ''}
                  </div>
                `).join('')}
              </div>
            </div>
            <div class="carousel-dots" id="carousel-dots">
              ${this.images.map((_, i) => `
                <div class="carousel-dot ${i === 0 ? 'active' : ''}" data-index="${i}"></div>
              `).join('')}
            </div>
          ` : `
            <div class="carousel-placeholder">
              <span>宣传图片即将上线</span>
            </div>
          `}
        </div>

        <!-- Enter Menu Button -->
        <div class="bottom-bar">
          <button class="enter-menu-btn" id="enter-menu-btn">
            \u{1F376} 点击查看产品
          </button>
        </div>
      </div>
    `;

    this.videoEl = document.getElementById('ad-video') as HTMLVideoElement;
    this.attachEventListeners();
  }

  private attachEventListeners(): void {
    // Enter menu button
    document.getElementById('enter-menu-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.cleanup();
      this.onEnterMenu();
    });

    // Carousel dots
    document.querySelectorAll('.carousel-dot').forEach(dot => {
      dot.addEventListener('click', (e) => {
        e.stopPropagation();
        const index = parseInt((dot as HTMLElement).dataset.index || '0');
        this.goToSlide(index);
      });
    });
  }

  // ---- Video ----

  private initVideo(): void {
    if (this.videos.length === 0 || !this.videoEl) return;

    this.videoEl.addEventListener('ended', () => {
      this.currentVideoIndex = (this.currentVideoIndex + 1) % this.videos.length;
      this.loadVideo(this.currentVideoIndex);
    });

    this.loadVideo(0);
  }

  private loadVideo(index: number): void {
    if (!this.videoEl) return;
    const video = this.videos[index];

    const titleEl = document.getElementById('video-title');
    if (titleEl) {
      titleEl.textContent = video.title || '';
    }

    // Safari native HLS
    if (this.videoEl.canPlayType('application/vnd.apple.mpegurl')) {
      this.destroyHls();
      this.videoEl.src = video.url;
      this.videoEl.play();
      return;
    }

    // hls.js for other browsers
    if (Hls.isSupported()) {
      this.destroyHls();
      this.hls = new Hls();
      this.hls.loadSource(video.url);
      this.hls.attachMedia(this.videoEl);
      this.hls.on(Hls.Events.MANIFEST_PARSED, () => {
        this.videoEl!.play();
      });
    }
  }

  private destroyHls(): void {
    if (this.hls) {
      this.hls.destroy();
      this.hls = null;
    }
  }

  // ---- Image Carousel ----

  private startCarousel(): void {
    if (this.images.length <= 1) return;
    const interval = this.images[0]?.duration || 5000;
    this.carouselTimer = window.setInterval(() => {
      this.carouselIndex = (this.carouselIndex + 1) % this.images.length;
      this.updateCarousel();
    }, interval);
  }

  private goToSlide(index: number): void {
    this.carouselIndex = index;
    this.updateCarousel();
    // Reset timer
    if (this.carouselTimer) {
      clearInterval(this.carouselTimer);
      this.startCarousel();
    }
  }

  private updateCarousel(): void {
    const track = document.getElementById('carousel-track');
    const dots = document.querySelectorAll('.carousel-dot');
    const slides = document.querySelectorAll('.carousel-slide');

    if (track) {
      track.style.transform = `translateX(-${this.carouselIndex * 100}%)`;
    }
    slides.forEach((slide, i) => {
      slide.classList.toggle('active', i === this.carouselIndex);
    });
    dots.forEach((dot, i) => {
      dot.classList.toggle('active', i === this.carouselIndex);
    });
  }

  // ---- Cleanup ----

  public cleanup(): void {
    this.destroyHls();
    if (this.videoEl) {
      this.videoEl.pause();
      this.videoEl.removeAttribute('src');
      this.videoEl.load();
    }
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
