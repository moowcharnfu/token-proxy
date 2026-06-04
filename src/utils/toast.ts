// Modern toast notification system
let toastElement: HTMLElement | null = null

export function showToast(message: string, type: 'success' | 'error' | 'info' = 'info', duration = 2500) {
  // Remove existing toast
  if (toastElement) {
    toastElement.remove()
  }

  // Create toast element
  toastElement = document.createElement('div')
  toastElement.className = `toast toast-${type}`
  toastElement.innerHTML = `
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      ${type === 'success'
        ? '<polyline points="20 6 9 17 4 12"></polyline>'
        : type === 'error'
        ? '<line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line>'
        : '<circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line>'
      }
    </svg>
    <span>${message}</span>
  `

  const colors = {
    success: { bg: 'var(--success)', glow: 'var(--success-glow)' },
    error: { bg: 'var(--error)', glow: 'var(--error-glow)' },
    info: { bg: 'var(--accent)', glow: 'var(--accent-glow)' }
  }
  const c = colors[type]

  toastElement.style.cssText = `
    position: fixed;
    bottom: 100px;
    left: 50%;
    transform: translateX(-50%) translateY(20px);
    padding: 12px 20px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 500;
    z-index: 9999;
    animation: toastIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) forwards;
    color: white;
    min-width: 220px;
    max-width: 400px;
    display: flex;
    align-items: center;
    gap: 8px;
    background: ${c.bg};
    box-shadow: 0 10px 40px ${c.glow}, 0 0 0 1px rgba(255,255,255,0.1);
  `

  document.body.appendChild(toastElement)

  // Auto remove
  setTimeout(() => {
    if (toastElement) {
      toastElement.style.animation = 'toastOut 0.3s cubic-bezier(0.4, 0, 0.2, 1) forwards'
      setTimeout(() => {
        toastElement?.remove()
        toastElement = null
      }, 300)
    }
  }, duration)
}
