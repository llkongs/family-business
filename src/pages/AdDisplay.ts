import { storeInfo, mediaPlaylist } from '../data/mockData';
import type { MediaItem } from '../data/mockData';

export class AdDisplay {
    private container: HTMLElement;
    private currentMediaIndex: number = 0;
    private carouselIndex: number = 0;
    private isPlayingCarousel: boolean = false;
    private carouselTimer: number | null = null;
    private onEnterMenu: () => void;

    constructor(container: HTMLElement, onEnterMenu: () => void) {
        this.container = container;
        this.onEnterMenu = onEnterMenu;
        this.render();
        this.startMediaPlayback();
    }

    private render(): void {
        const images = mediaPlaylist.filter(m => m.type === 'image');

        this.container.innerHTML = `
      <div class="ad-display">
        <!-- Header Bar -->
        <div class="header-bar">
          <div class="store-info">
            <span class="store-name">🏪 ${storeInfo.name}</span>
            <span class="store-phone">${storeInfo.phone}</span>
          </div>
          <div class="qr-code">
            <img src="${storeInfo.qrCodeUrl}" alt="QR Code" />
          </div>
        </div>

        <!-- Video Section -->
        <div class="video-section">
          <video 
            class="video-player" 
            id="video-player"
            muted 
            playsinline
            poster="https://images.unsplash.com/photo-1504674900247-0877df9cc836?w=800&h=600&fit=crop"
          >
            <source src="" type="video/mp4" />
          </video>
          <div class="video-overlay" id="video-overlay">
            <div class="loading">
              <div class="spinner"></div>
            </div>
          </div>
        </div>

        <!-- Carousel Section -->
        <div class="carousel-section" id="carousel-section">
          <div class="carousel-container">
            <div class="carousel-track" id="carousel-track">
              ${images.map(img => `
                <div class="carousel-slide">
                  <img src="${img.url}" alt="${img.title || ''}" />
                </div>
              `).join('')}
            </div>
          </div>
          <div class="carousel-dots" id="carousel-dots">
            ${images.map((_, i) => `
              <div class="carousel-dot ${i === 0 ? 'active' : ''}" data-index="${i}"></div>
            `).join('')}
          </div>
          <button class="enter-menu-btn" id="enter-menu-btn">
            👆 点击查看菜单
          </button>
        </div>
      </div>
    `;

        this.attachEventListeners();
    }

    private attachEventListeners(): void {
        const videoPlayer = document.getElementById('video-player') as HTMLVideoElement;
        const enterMenuBtn = document.getElementById('enter-menu-btn');
        const carouselSection = document.getElementById('carousel-section');
        const dots = document.querySelectorAll('.carousel-dot');

        // Video ended event
        videoPlayer?.addEventListener('ended', () => {
            this.onVideoEnded();
        });

        // Video error - skip to next
        videoPlayer?.addEventListener('error', () => {
            console.log('Video error, starting carousel...');
            this.startCarousel();
        });

        // Enter menu button
        enterMenuBtn?.addEventListener('click', (e) => {
            e.stopPropagation();
            this.cleanup();
            this.onEnterMenu();
        });

        // Click on carousel area
        carouselSection?.addEventListener('click', (e) => {
            if ((e.target as HTMLElement).id !== 'enter-menu-btn') {
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
    }

    private startMediaPlayback(): void {
        const videos = mediaPlaylist.filter(m => m.type === 'video');

        if (videos.length > 0) {
            this.playVideo(videos[0]);
        } else {
            this.startCarousel();
        }
    }

    private playVideo(media: MediaItem): void {
        const videoPlayer = document.getElementById('video-player') as HTMLVideoElement;
        const overlay = document.getElementById('video-overlay');

        if (videoPlayer && media.url) {
            overlay?.classList.remove('hidden');
            videoPlayer.src = media.url;

            videoPlayer.oncanplay = () => {
                overlay?.classList.add('hidden');
                videoPlayer.play().catch(() => {
                    console.log('Autoplay blocked, starting carousel...');
                    this.startCarousel();
                });
            };

            // Timeout fallback
            setTimeout(() => {
                if (videoPlayer.paused) {
                    console.log('Video not playing, starting carousel...');
                    this.startCarousel();
                }
            }, 5000);
        }
    }

    private onVideoEnded(): void {
        // After video ends, start carousel
        this.startCarousel();
    }

    private startCarousel(): void {
        if (this.isPlayingCarousel) return;

        this.isPlayingCarousel = true;
        this.carouselIndex = 0;
        this.updateCarouselPosition();

        const images = mediaPlaylist.filter(m => m.type === 'image');
        const interval = images[0]?.duration || 4000;

        this.carouselTimer = window.setInterval(() => {
            this.carouselIndex = (this.carouselIndex + 1) % images.length;
            this.updateCarouselPosition();

            // After one full cycle, try to play video again
            if (this.carouselIndex === 0) {
                this.stopCarousel();
                const videos = mediaPlaylist.filter(m => m.type === 'video');
                if (videos.length > 0) {
                    this.currentMediaIndex = (this.currentMediaIndex + 1) % videos.length;
                    this.playVideo(videos[this.currentMediaIndex]);
                } else {
                    // No videos, restart carousel
                    setTimeout(() => this.startCarousel(), 1000);
                }
            }
        }, interval);
    }

    private stopCarousel(): void {
        if (this.carouselTimer) {
            clearInterval(this.carouselTimer);
            this.carouselTimer = null;
        }
        this.isPlayingCarousel = false;
    }

    private goToSlide(index: number): void {
        this.carouselIndex = index;
        this.updateCarouselPosition();
    }

    private updateCarouselPosition(): void {
        const track = document.getElementById('carousel-track');
        const dots = document.querySelectorAll('.carousel-dot');

        if (track) {
            track.style.transform = `translateX(-${this.carouselIndex * 100}%)`;
        }

        dots.forEach((dot, i) => {
            dot.classList.toggle('active', i === this.carouselIndex);
        });
    }

    public cleanup(): void {
        this.stopCarousel();
        const videoPlayer = document.getElementById('video-player') as HTMLVideoElement;
        if (videoPlayer) {
            videoPlayer.pause();
            videoPlayer.src = '';
        }
    }

    public destroy(): void {
        this.cleanup();
        this.container.innerHTML = '';
    }
}
