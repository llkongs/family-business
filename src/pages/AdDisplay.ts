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

  constructor(container: HTMLElement, onEnterMenu: () => void) {
    this.container = container;
    this.onEnterMenu = onEnterMenu;
    this.render();
    this.initVideo();
  }

  private render(): void {
    const hasImages = this.images.length > 0;

    this.container.innerHTML = `
      <div class="ad-display${hasImages ? '' : ' no-images'}">
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
            <button class="enter-menu-btn" id="enter-menu-btn">
              \u{1F376} 菜单
            </button>
          </div>
        </div>

        <!-- Video Section -->
        <div class="video-section">
          <video id="ad-video" muted playsinline></video>
          <div class="video-title" id="video-title"></div>
        </div>

        <!-- Image Carousel -->
        ${hasImages ? `
        <div class="image-carousel">
          <div class="image-track" id="image-track">
            ${this.images.map(img => `<img src="${img.url}" alt="${img.title || ''}" />`).join('')}
            ${this.images.map(img => `<img src="${img.url}" alt="${img.title || ''}" />`).join('')}
          </div>
        </div>
        ` : ''}
      </div>
    `;

    this.videoEl = document.getElementById('ad-video') as HTMLVideoElement;
    this.attachEventListeners();
    this.setImageScrollSpeed();
  }

  private attachEventListeners(): void {
    document.getElementById('enter-menu-btn')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.cleanup();
      this.onEnterMenu();
    });
  }

  private setImageScrollSpeed(): void {
    const track = document.getElementById('image-track') as HTMLElement | null;
    if (!track || this.images.length === 0) return;
    // ~15s per image for a smooth scroll
    const duration = this.images.length * 15;
    track.style.animationDuration = `${duration}s`;
  }

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

    // Update title
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

    // Use hls.js for other browsers
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

  public cleanup(): void {
    this.destroyHls();
    if (this.videoEl) {
      this.videoEl.pause();
      this.videoEl.removeAttribute('src');
      this.videoEl.load();
    }
  }

  public destroy(): void {
    this.cleanup();
    this.container.innerHTML = '';
  }
}
