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
        <div style="background:#14141f; border:1px solid rgba(233,69,96,0.3); border-radius:16px; padding:2rem; max-width:480px; width:100%; position:relative;">
          <button onclick="document.getElementById('quick-add-modal').remove()" style="position:absolute;top:1rem;right:1rem;background:none;border:none;color:#a0a0b0;font-size:1.4rem;cursor:pointer;">✕</button>
          <h3 style="font-family:'Playfair Display',serif;font-size:1.4rem;margin-bottom:0.25rem;">${name}</h3>
          <div style="font-family:'Space Mono',monospace;color:#e94560;font-size:1.2rem;margin-bottom:1.5rem;">₹${Math.round(price)}</div>
          
          <div style="margin-bottom:1.25rem;">
            <div style="font-family:'Space Mono',monospace;font-size:0.7rem;letter-spacing:2px;text-transform:uppercase;color:#a0a0b0;margin-bottom:0.75rem;">Select Size</div>
            <div id="modal-sizes" style="display:flex;flex-wrap:wrap;gap:0.5rem;">
              ${(p.sizes || []).map(s => `
                <button onclick="modalSelectSize('${s}')" data-size="${s}"
                  style="min-width:44px;height:44px;padding:0 0.75rem;border-radius:8px;border:1px solid rgba(233,69,96,0.2);background:#1a1a2e;color:#f0f0f0;font-family:'Space Mono',monospace;font-size:0.8rem;cursor:pointer;transition:all 0.2s;">
                  ${s}
                </button>
              `).join('')}
            </div>
          </div>
          
          <div style="margin-bottom:1.75rem;">
            <div style="font-family:'Space Mono',monospace;font-size:0.7rem;letter-spacing:2px;text-transform:uppercase;color:#a0a0b0;margin-bottom:0.75rem;">Select Color</div>
            <div id="modal-colors" style="display:flex;flex-wrap:wrap;gap:0.5rem;">
              ${(p.colors || []).map((c, i) => `
                <button onclick="modalSelectColor('${c}')" data-color="${c}"
                  style="padding:0.4rem 0.9rem;border-radius:6px;border:1px solid ${i===0?'#e94560':'rgba(233,69,96,0.2)'};background:${i===0?'rgba(233,69,96,0.15)':'#1a1a2e'};color:${i===0?'#e94560':'#a0a0b0'};font-size:0.8rem;cursor:pointer;transition:all 0.2s;">
                  ${c}
                </button>
              `).join('')}
            </div>
          </div>

          <button onclick="modalAddToCart(${id}, '${name}')" style="width:100%;padding:1rem;background:#e94560;color:white;border:none;border-radius:12px;font-size:0.85rem;font-family:'DM Sans',sans-serif;letter-spacing:1.5px;text-transform:uppercase;cursor:pointer;font-weight:500;transition:all 0.2s;">
            Add to Cart
          </button>
          <a href="/product/${id}" style="display:block;text-align:center;margin-top:0.75rem;font-size:0.8rem;color:#a0a0b0;text-decoration:underline;">View Full Details →</a>
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
    b.style.background = isActive ? '#e94560' : '#1a1a2e';
    b.style.borderColor = isActive ? '#e94560' : 'rgba(233,69,96,0.2)';
    b.style.color = isActive ? 'white' : '#f0f0f0';
  });
}

function modalSelectColor(color) {
  window._modalSelectedColor = color;
  document.querySelectorAll('#modal-colors button').forEach(b => {
    const isActive = b.dataset.color === color;
    b.style.background = isActive ? 'rgba(233,69,96,0.15)' : '#1a1a2e';
    b.style.borderColor = isActive ? '#e94560' : 'rgba(233,69,96,0.2)';
    b.style.color = isActive ? '#e94560' : '#a0a0b0';
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
