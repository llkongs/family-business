import Hls from 'hls.js';
import { storeInfo, mediaPlaylist, slogans } from '../data/mockData';

// T015: HTML escape helper to prevent XSS from external data
function esc(s: string): string {
  const el = document.createElement('span');
  el.textContent = s;
  return el.innerHTML;
}

function safeUrlValue(url: string): string {
  const raw = url.trim();
  if (!raw) return '';
  if (
    raw.startsWith('http://') ||
    raw.startsWith('https://') ||
    raw.startsWith('/') ||
    raw.startsWith('./') ||
    raw.startsWith('../')
  ) {
    return raw;
  }
  return '';
}

export class AdDisplay {
  private container: HTMLElement;
  private onEnterMenu: (() => void) | null;
  private hls: Hls | null = null;
  private videoEl: HTMLVideoElement | null = null;
  private videos = mediaPlaylist.filter(m => m.type === 'video');
  private images = mediaPlaylist.filter(m => m.type === 'image');
  private currentVideoIndex = 0;
  private carouselIndex = 0;
  private carouselTimer: number | null = null;
  private watchdogTimer: number | null = null;
  private lastCurrentTime = -1;
  private stallCount = 0;
  private static readonly WATCHDOG_INTERVAL = 5000;
  private static readonly MAX_STALL_COUNT = 3;

  constructor(container: HTMLElement, onEnterMenu: (() => void) | null) {
    this.container = container;
    this.onEnterMenu = onEnterMenu;
    this.render();
    this.initVideo();
    this.startCarousel();
  }

  private formatPhone(phone: string): string {
    const digits = phone.replace(/\D/g, '');
    if (digits.length === 11) {
      return `${digits.slice(0, 3)}-${digits.slice(3, 7)}-${digits.slice(7)}`;
    }
    return phone;
  }

  private render(): void {
    const hasImages = this.images.length > 0;
    const showCta = this.onEnterMenu !== null;

    this.container.innerHTML = `
      <div class="ad-display">
        <!-- Header Bar -->
        <div class="header-bar">
          <div class="store-info">
            <p class="store-eyebrow">SHAOXING HUANGJIU</p>
            <span class="store-name">${esc(storeInfo.name)}</span>
            <p class="store-phone">
              <span class="phone-tag">服务热线</span>
              <strong class="phone-num">${esc(this.formatPhone(storeInfo.phone))}</strong>
            </p>
          </div>
          <div class="header-right">
            <div class="qr-code">
              <img src="${esc(safeUrlValue(storeInfo.qrCodeUrl))}" alt="QR Code" />
            </div>
            <p class="qr-caption">扫码咨询</p>
          </div>
        </div>

        <!-- Video Section -->
        <div class="video-section">
          <video id="ad-video" playsinline></video>
          <button class="unmute-btn hidden" id="unmute-btn">\u{1F50A} \u70B9\u51FB\u5F00\u542F\u58F0\u97F3</button>
        </div>

        <!-- Marquee Ticker -->
        ${slogans.length > 0 ? `
        <div class="ticker-bar">
          <div class="ticker-track">
            ${slogans.map(s => `<span class="ticker-item">${esc(s.text)}</span>`).join('')}
            ${slogans.map(s => `<span class="ticker-item">${esc(s.text)}</span>`).join('')}
          </div>
        </div>
        ` : ''}

        <!-- Image Carousel -->
        <div class="image-carousel">
          ${hasImages ? `
            <div class="carousel-container">
              <div class="carousel-track" id="carousel-track">
                ${this.images.map((img, i) => `
                  <div class="carousel-slide ${i === 0 ? 'active' : ''}" data-index="${i}">
                    <img src="${esc(safeUrlValue(img.url))}" alt="${esc(img.title || '')}" />
                    ${img.title ? `<div class="carousel-caption">
                      <div class="carousel-caption-title">${esc(img.title)}</div>
                      ${img.description ? `<p class="carousel-caption-desc">${esc(img.description)}</p>` : ''}
                    </div>` : ''}
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
              <span>\u{1F3BA} \u5BA3\u4F20\u56FE\u7247\u5373\u5C06\u4E0A\u7EBF</span>
            </div>
          `}

          ${showCta ? `
            <button class="cta-fab" id="enter-menu-btn">
              查看产品详情
              <svg viewBox="0 0 24 24" fill="none" width="1em" height="1em"><path d="M5 12h14M13 6l6 6-6 6" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
          ` : ''}
        </div>
      </div>
    `;

    this.videoEl = document.getElementById('ad-video') as HTMLVideoElement;
    this.attachEventListeners();
  }

  private attachEventListeners(): void {
    document.getElementById('enter-menu-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.cleanup();
      this.onEnterMenu?.();
    });

    document.getElementById('unmute-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      if (this.videoEl) {
        this.videoEl.muted = false;
      }
      (e.currentTarget as HTMLElement).classList.add('hidden');
    });

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
    this.startWatchdog();
  }

  private loadVideo(index: number): void {
    if (!this.videoEl) return;
    const video = this.videos[index];
    const source = safeUrlValue(video.url);
    if (!source) {
      console.error('[Video] Invalid source URL, skipping:', video.url);
      this.skipToNextVideo();
      return;
    }

    this.lastCurrentTime = -1;
    this.stallCount = 0;

    // Safari native HLS
    if (this.videoEl.canPlayType('application/vnd.apple.mpegurl')) {
      this.destroyHls();
      this.videoEl.src = source;
      this.tryPlay();
      return;
    }

    // hls.js for other browsers
    if (Hls.isSupported()) {
      this.destroyHls();
      this.hls = new Hls({
        enableWorker: true,
        lowLatencyMode: false,
      });
      this.hls.loadSource(source);
      this.hls.attachMedia(this.videoEl);

      // T007: HLS error recovery
      this.hls.on(Hls.Events.ERROR, (_event, data) => {
        if (!data.fatal) return;
        switch (data.type) {
          case Hls.ErrorTypes.NETWORK_ERROR:
            console.warn('[HLS] Fatal network error, attempting recovery...');
            this.hls?.startLoad();
            break;
          case Hls.ErrorTypes.MEDIA_ERROR:
            console.warn('[HLS] Fatal media error, attempting recovery...');
            this.hls?.recoverMediaError();
            break;
          default:
            console.error('[HLS] Unrecoverable error, skipping to next video');
            this.skipToNextVideo();
            break;
        }
      });

      this.hls.on(Hls.Events.MANIFEST_PARSED, () => {
        this.tryPlay();
      });
    }
  }

  // T006: play() with catch — muted fallback + show unmute button
  private tryPlay(): void {
    if (!this.videoEl) return;
    this.videoEl.play().catch(() => {
      console.warn('[Video] Autoplay blocked, retrying muted...');
      if (this.videoEl) {
        this.videoEl.muted = true;
        this.videoEl.play().then(() => {
          document.getElementById('unmute-btn')?.classList.remove('hidden');
        }).catch((err) => {
          console.error('[Video] Muted autoplay also failed:', err);
        });
      }
    });
  }

  private skipToNextVideo(): void {
    this.currentVideoIndex = (this.currentVideoIndex + 1) % this.videos.length;
    this.loadVideo(this.currentVideoIndex);
  }

  private destroyHls(): void {
    if (this.hls) {
      this.hls.destroy();
      this.hls = null;
    }
  }

  // T008: Watchdog — detect stalled playback
  private startWatchdog(): void {
    this.watchdogTimer = window.setInterval(() => {
      if (!this.videoEl || this.videoEl.paused) return;
      const ct = this.videoEl.currentTime;
      if (this.lastCurrentTime >= 0 && ct === this.lastCurrentTime) {
        this.stallCount++;
        console.warn(`[Watchdog] Stall detected (${this.stallCount}/${AdDisplay.MAX_STALL_COUNT})`);
        if (this.stallCount >= AdDisplay.MAX_STALL_COUNT) {
          console.error('[Watchdog] Max stalls reached, skipping to next video');
          this.skipToNextVideo();
        }
      } else {
        this.stallCount = 0;
      }
      this.lastCurrentTime = ct;
    }, AdDisplay.WATCHDOG_INTERVAL);
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
    if (this.watchdogTimer) {
      clearInterval(this.watchdogTimer);
      this.watchdogTimer = null;
    }
  }

  public destroy(): void {
    this.cleanup();
    this.container.innerHTML = '';
  }
}
