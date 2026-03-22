// ============================================================
// ThreadCraft — Main JavaScript
// ============================================================

// ---- Nav scroll effect ----
window.addEventListener('scroll', () => {
  const nav = document.getElementById('nav');
  if (nav) {
    if (window.scrollY > 20) {
      nav.classList.add('scrolled');
    } else {
      nav.classList.remove('scrolled');
    }
  }
});

// ---- Active nav link — highlight based on current URL ----
(function() {
  const path = window.location.pathname;
  const search = window.location.search;
  document.querySelectorAll('.nav-links a, .nav-mobile a').forEach(a => {
    a.classList.remove('active');
    const href = a.getAttribute('href');
    if (!href) return;
    // Exact match or starts with (for /shop?category=...)
    if (href === path || (href !== '/' && path.startsWith(href) && href.length > 1)) {
      a.classList.add('active');
    }
    // Handle category links e.g. /shop?category=T-Shirts
    if (href.includes('?') && (path + search) === href) {
      a.classList.add('active');
    }
    if (href === '/shop' && path === '/shop' && !search) {
      a.classList.add('active');
    }
  });
})();

// ---- Close search when clicking outside ----
document.addEventListener('click', (e) => {
  const bar = document.getElementById('nav-search-bar');
  const btn = document.querySelector('.nav-search-btn');
  if (bar && bar.classList.contains('open')) {
    if (!bar.contains(e.target) && e.target !== btn && !btn?.contains(e.target)) {
      bar.classList.remove('open');
    }
  }
});

// ---- Mobile menu ----
function toggleMenu() {
  const menu = document.getElementById('nav-mobile');
  if (menu) menu.classList.toggle('open');
}

// ---- Toast notifications ----
let toastTimer;
function showToast(msg, type = 'success') {
  const toast = document.getElementById('toast');
  if (!toast) return;
  clearTimeout(toastTimer);
  toast.textContent = msg;
  toast.className = `toast ${type} show`;
  toastTimer = setTimeout(() => {
    toast.classList.remove('show');
  }, 3000);
}

// ---- Quick Add Modal ----
function quickAdd(id, name, price, img) {
  showQuickAddModal(id, name, price, img);
}

function showQuickAddModal(id, name, price, img) {
  // Remove existing
  const existing = document.getElementById('quick-add-modal');
  if (existing) existing.remove();

  // Fetch product details for sizes/colors
  fetch(`/api/products/${id}`)
    .then(r => r.json())
    .then(data => {
      if (!data.success) return;
      const p = data.data;
      const modal = document.createElement('div');
      modal.id = 'quick-add-modal';
      modal.style.cssText = `
        position: fixed; inset: 0; z-index: 200;
        background: rgba(0,0,0,0.8); backdrop-filter: blur(8px);
        display: flex; align-items: center; justify-content: center;
        padding: 1rem;
      `;
      modal.innerHTML = `
        <div style="background:#fff;border:3px solid #0a0a0a;border-radius:16px;padding:0;max-width:520px;width:100%;position:relative;overflow:hidden;box-shadow:8px 8px 0 #0a0a0a;">
          <div style="display:flex;gap:0;max-height:85vh;">
            <div style="width:180px;flex-shrink:0;background:#f8f8f8;border-right:3px solid #0a0a0a;overflow:hidden;">
              <img src="${img}" alt="${name}" style="width:100%;height:100%;object-fit:cover;object-position:top center;"/>
            </div>
            <div style="flex:1;padding:1.75rem;overflow-y:auto;">
              <button onclick="document.getElementById('quick-add-modal').remove()" style="position:absolute;top:1rem;right:1rem;background:none;border:2px solid #0a0a0a;border-radius:6px;width:32px;height:32px;font-size:1rem;cursor:pointer;font-weight:900;display:flex;align-items:center;justify-content:center;">✕</button>
              <p style="font-family:'JetBrains Mono',monospace;font-size:0.62rem;letter-spacing:0.18em;text-transform:uppercase;color:#888;margin-bottom:0.4rem;">Quick Add</p>
              <h3 style="font-family:'Bebas Neue',sans-serif;font-size:1.8rem;letter-spacing:0.04em;margin-bottom:0.25rem;line-height:1;color:#0a0a0a;">${name}</h3>
              <div style="font-family:'Bebas Neue',sans-serif;color:#e63000;font-size:1.6rem;margin-bottom:1.5rem;letter-spacing:0.04em;">₹${Math.round(price)}</div>
              
              <div style="margin-bottom:1.25rem;">
                <div style="font-family:'JetBrains Mono',monospace;font-size:0.62rem;letter-spacing:0.18em;text-transform:uppercase;color:#888;margin-bottom:0.65rem;font-weight:700;">Select Size</div>
                <div id="modal-sizes" style="display:flex;flex-wrap:wrap;gap:0.4rem;">
                  ${(p.sizes || []).map(s => `
                    <button onclick="modalSelectSize('${s}')" data-size="${s}"
                      style="min-width:46px;height:46px;padding:0 0.75rem;border-radius:6px;border:2px solid #0a0a0a;background:#fff;color:#0a0a0a;font-family:'Inter',sans-serif;font-size:0.8rem;font-weight:800;cursor:pointer;transition:all 0.2s;">
                      ${s}
                    </button>
                  `).join('')}
                </div>
              </div>
              
              <div style="margin-bottom:1.75rem;">
                <div style="font-family:'JetBrains Mono',monospace;font-size:0.62rem;letter-spacing:0.18em;text-transform:uppercase;color:#888;margin-bottom:0.65rem;font-weight:700;">Select Color</div>
                <div id="modal-colors" style="display:flex;flex-wrap:wrap;gap:0.4rem;">
                  ${(p.colors || []).map((clr, i) => `
                    <button onclick="modalSelectColor('${clr}')" data-color="${clr}"
                      style="padding:0.45rem 1rem;border-radius:6px;border:2px solid ${i===0?'#e63000':'#d0d0d0'};background:${i===0?'#e63000':'#fff'};color:${i===0?'#fff':'#444'};font-size:0.78rem;font-weight:700;cursor:pointer;transition:all 0.2s;font-family:'Inter',sans-serif;">
                      ${clr}
                    </button>
                  `).join('')}
                </div>
              </div>

              <button onclick="modalAddToCart(${id}, '${name}')" style="width:100%;padding:1rem;background:#e63000;color:white;border:none;border-radius:6px;font-size:0.78rem;font-family:'Inter',sans-serif;letter-spacing:0.12em;text-transform:uppercase;cursor:pointer;font-weight:800;transition:all 0.2s;border:2px solid #e63000;">
                ADD TO CART
              </button>
              <a href="/product/${id}" style="display:block;text-align:center;margin-top:0.75rem;font-size:0.78rem;color:#888;font-family:'JetBrains Mono',monospace;font-weight:500;">View Full Details →</a>
            </div>
          </div>
        </div>
      `;
      document.body.appendChild(modal);
      
      // Set first color as active
      window._modalSelectedColor = (p.colors || [])[0] || '';
      window._modalSelectedSize = null;
      
      modal.addEventListener('click', (e) => { if (e.target === modal) modal.remove(); });
    });
}

function modalSelectSize(size) {
  window._modalSelectedSize = size;
  document.querySelectorAll('#modal-sizes button').forEach(b => {
    const isActive = b.dataset.size === size;
    b.style.background = isActive ? '#0a0a0a' : '#fff';
    b.style.borderColor = '#0a0a0a';
    b.style.color = isActive ? 'white' : '#0a0a0a';
  });
}

function modalSelectColor(color) {
  window._modalSelectedColor = color;
  document.querySelectorAll('#modal-colors button').forEach(b => {
    const isActive = b.dataset.color === color;
    b.style.background = isActive ? '#e63000' : '#fff';
    b.style.borderColor = isActive ? '#e63000' : '#d0d0d0';
    b.style.color = isActive ? 'white' : '#444';
  });
}

async function modalAddToCart(id, name) {
  if (!window._modalSelectedSize) {
    showToast('Please select a size', 'error');
    return;
  }
  const res = await fetch('/api/cart/add', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      product_id: id,
      size: window._modalSelectedSize,
      color: window._modalSelectedColor,
      quantity: 1
    })
  });
  const data = await res.json();
  if (data.success) {
    const el = document.getElementById('cart-count');
    if (el) el.textContent = data.cart_count;
    document.getElementById('quick-add-modal')?.remove();
    showToast(`${name} added to cart! ✓`);
  }
}

// ---- Animate on scroll ----
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.style.opacity = '1';
      entry.target.style.transform = 'translateY(0)';
    }
  });
}, { threshold: 0.1 });

document.addEventListener('DOMContentLoaded', () => {
  // Animate cards
  document.querySelectorAll('.product-card, .usp-item, .cat-card').forEach((el, i) => {
    el.style.opacity = '0';
    el.style.transform = 'translateY(24px)';
    el.style.transition = `opacity 0.5s ease ${i * 0.06}s, transform 0.5s ease ${i * 0.06}s`;
    observer.observe(el);
  });
});

// ============================================================
// SEARCH
// ============================================================
function toggleSearch() {
  const bar = document.getElementById('nav-search-bar');
  bar.classList.toggle('open');
  if (bar.classList.contains('open')) {
    bar.querySelector('input')?.focus();
  }
}

// Close search on Escape
document.addEventListener('keydown', e => {
  if (e.key === 'Escape') {
    document.getElementById('nav-search-bar')?.classList.remove('open');
    document.getElementById('wishlist-drawer')?.classList.remove('open');
    document.getElementById('wishlist-overlay')?.classList.remove('open');
  }
});

// ============================================================
// WISHLIST (localStorage)
// ============================================================
function getWishlist() {
  return JSON.parse(localStorage.getItem('tc_wishlist') || '[]');
}
function saveWishlist(list) {
  localStorage.setItem('tc_wishlist', JSON.stringify(list));
}

function toggleWishlistItem(product) {
  let list = getWishlist();
  const idx = list.findIndex(p => p.id === product.id);
  if (idx === -1) {
    list.push(product);
    showToast(`❤️ ${product.name} added to wishlist!`);
  } else {
    list.splice(idx, 1);
    showToast(`Removed from wishlist`);
  }
  saveWishlist(list);
  updateWishlistBtns();
  if (document.getElementById('wishlist-drawer')?.classList.contains('open')) {
    renderWishlistDrawer();
  }
}

function updateWishlistBtns() {
  const list = getWishlist();
  document.querySelectorAll('[data-wish-id]').forEach(btn => {
    const id = parseInt(btn.dataset.wishId);
    const wishlisted = list.some(p => p.id === id);
    btn.classList.toggle('wishlisted', wishlisted);
    btn.title = wishlisted ? 'Remove from Wishlist' : 'Add to Wishlist';
    btn.textContent = wishlisted ? '❤️' : '🤍';
  });
}

function toggleWishlist() {
  const drawer = document.getElementById('wishlist-drawer');
  const overlay = document.getElementById('wishlist-overlay');
  if (!drawer) {
    // Create the drawer
    const drawerEl = document.createElement('div');
    drawerEl.id = 'wishlist-drawer';
    drawerEl.className = 'wishlist-drawer';
    drawerEl.innerHTML = `
      <div class="drawer-header">
        <h3>Wishlist ❤️</h3>
        <button class="drawer-close" onclick="closeWishlist()">✕</button>
      </div>
      <div class="drawer-body" id="wishlist-body"></div>
      <div class="drawer-footer">
        <a href="/shop" class="btn btn-outline btn-full">Browse More Products</a>
      </div>
    `;
    const overlayEl = document.createElement('div');
    overlayEl.id = 'wishlist-overlay';
    overlayEl.className = 'wishlist-overlay';
    overlayEl.onclick = closeWishlist;
    document.body.appendChild(overlayEl);
    document.body.appendChild(drawerEl);
    setTimeout(() => {
      drawerEl.classList.add('open');
      overlayEl.classList.add('open');
    }, 10);
  } else {
    drawer.classList.toggle('open');
    document.getElementById('wishlist-overlay')?.classList.toggle('open');
  }
  renderWishlistDrawer();
}

function closeWishlist() {
  document.getElementById('wishlist-drawer')?.classList.remove('open');
  document.getElementById('wishlist-overlay')?.classList.remove('open');
}

function renderWishlistDrawer() {
  const body = document.getElementById('wishlist-body');
  if (!body) return;
  const list = getWishlist();
  if (!list.length) {
    body.innerHTML = `<div class="empty-wishlist"><div class="icon">🤍</div><p>No saved items yet.<br/>Click 🤍 on any product to save it.</p></div>`;
    return;
  }
  body.innerHTML = list.map(p => `
    <div class="wishlist-item">
      <div class="wishlist-item-img">
        <svg viewBox="0 0 60 68" xmlns="http://www.w3.org/2000/svg">
          <path d="M15,6 L6,17 L14,20 L12,60 L48,60 L46,20 L54,17 L45,6 C42,11 40,13 30,13 C20,13 18,11 15,6Z" fill="#1a1a2e" stroke="#e94560" stroke-width="0.7"/>
        </svg>
      </div>
      <div class="wishlist-item-info">
        <strong>${p.name}</strong>
        <span>₹${Math.round(p.price)}</span>
      </div>
      <a href="/product/${p.id}" class="btn btn-ghost btn-sm">Buy</a>
      <button class="wishlist-remove" onclick='toggleWishlistItem(${JSON.stringify(p)})' title="Remove">✕</button>
    </div>
  `).join('');
}

// ============================================================
// Inject wishlist buttons on product cards
// ============================================================
document.addEventListener('DOMContentLoaded', () => {
  // Add wishlist heart buttons to product cards
  document.querySelectorAll('.product-card[data-id]').forEach(card => {
    const id = parseInt(card.dataset.id);
    const name = card.querySelector('.product-name')?.textContent?.trim() || '';
    const priceText = card.querySelector('.product-price')?.textContent?.replace('₹','') || '0';
    const price = parseFloat(priceText);

    // Only add if image-wrap exists
    const wrap = card.querySelector('.product-image-wrap');
    if (wrap && !card.querySelector('[data-wish-id]')) {
      const btn = document.createElement('button');
      btn.className = 'product-wish-btn';
      btn.dataset.wishId = id;
      btn.title = 'Add to Wishlist';
      btn.textContent = '🤍';
      btn.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        toggleWishlistItem({ id, name, price });
      };
      wrap.style.position = 'relative';
      wrap.appendChild(btn);
    }
  });

  updateWishlistBtns();
});

// ============================================================
// HERO SLIDESHOW — Stack Slide (vertical flip)
// ============================================================
(function() {
  const names = [
    "Hanuman — The Devoted Warrior",
    "Durga — The Invincible",
    "Kali — Dark Mother",
    "Arjuna & Krishna — Kurukshetra",
    "Poseidon — Lord of the Seas",
    "Athena — Goddess of Wisdom",
    "Hades — King of the Underworld",
    "Anubis — Guardian of the Dead",
    "Quetzalcoatl — The Feathered Serpent",
    "Nezha — Lotus Prince",
    "Susanoo — The Storm God",
    "Thor — God of Thunder",
    "Quetzalcoatl II — Serpent God"
  ];

  const container = document.getElementById('heroSlideshow');
  if (!container) return;

  const slides = Array.from(container.querySelectorAll('.slide'));
  const dotsEl = document.getElementById('slideshowDots');
  const labelEl = document.getElementById('slideshowLabel');
  let current = 0;
  let timer = null;
  let isAnimating = false;

  // Build dots
  slides.forEach((_, i) => {
    const d = document.createElement('button');
    d.className = 'sdot' + (i === 0 ? ' active' : '');
    d.setAttribute('aria-label', 'Slide ' + (i+1));
    d.onclick = () => goTo(i);
    dotsEl.appendChild(d);
  });

  // Build nav arrows
  const nav = document.createElement('div');
  nav.className = 'slideshow-nav';
  nav.innerHTML = '<button class="snav-btn" id="snav-prev">&#8593;</button><button class="snav-btn" id="snav-next">&#8595;</button>';
  container.appendChild(nav);
  document.getElementById('snav-prev').onclick = () => goTo((current - 1 + slides.length) % slides.length);
  document.getElementById('snav-next').onclick = () => goTo((current + 1) % slides.length);

  function updateDots(idx) {
    dotsEl.querySelectorAll('.sdot').forEach((d, i) => {
      d.classList.toggle('active', i === idx);
    });
  }

  function updateLabel(idx) {
    if (!labelEl) return;
    labelEl.style.opacity = '0';
    labelEl.style.transform = 'translateY(8px)';
    labelEl.style.transition = 'opacity 0.3s ease, transform 0.3s ease';
    setTimeout(() => {
      labelEl.textContent = names[idx] || '';
      labelEl.style.opacity = '1';
      labelEl.style.transform = 'translateY(0)';
    }, 300);
  }

  function goTo(next) {
    if (isAnimating || next === current) return;
    isAnimating = true;

    const curr = slides[current];
    const nextSlide = slides[next];

    // Position next slide below (or above if going backwards)
    const goingDown = next > current || (current === slides.length - 1 && next === 0);

    nextSlide.style.transition = 'none';
    nextSlide.style.transform = goingDown ? 'translateY(110%)' : 'translateY(-110%)';
    nextSlide.style.zIndex = '1';

    // Force reflow
    nextSlide.getBoundingClientRect();

    // Animate current out
    curr.style.transition = 'transform 0.65s cubic-bezier(0.76,0,0.24,1)';
    curr.style.transform = goingDown ? 'translateY(-110%)' : 'translateY(110%)';
    curr.style.zIndex = '3';

    // Animate next in
    nextSlide.style.transition = 'transform 0.65s cubic-bezier(0.76,0,0.24,1)';
    nextSlide.style.transform = 'translateY(0)';
    nextSlide.style.zIndex = '2';

    updateDots(next);
    updateLabel(next);

    setTimeout(() => {
      curr.style.transition = 'none';
      curr.style.transform = 'translateY(100%)';
      curr.style.zIndex = '1';
      current = next;
      isAnimating = false;
    }, 680);
  }

  function startTimer() {
    timer = setInterval(() => {
      goTo((current + 1) % slides.length);
    }, 3500);
  }

  function stopTimer() { clearInterval(timer); }

  // Pause on hover
  container.addEventListener('mouseenter', stopTimer);
  container.addEventListener('mouseleave', startTimer);

  // Touch swipe support
  let touchStartY = 0;
  container.addEventListener('touchstart', e => { touchStartY = e.touches[0].clientY; }, {passive:true});
  container.addEventListener('touchend', e => {
    const diff = touchStartY - e.changedTouches[0].clientY;
    if (Math.abs(diff) > 40) goTo(diff > 0 ? (current+1) % slides.length : (current-1+slides.length) % slides.length);
  }, {passive:true});

  // Set initial label style
  if (labelEl) {
    labelEl.style.opacity = '1';
    labelEl.style.transform = 'translateY(0)';
  }

  startTimer();
})();
