(function () {
  const ICONS = [
    {
      href: "https://x.com/aethr_ai",
      label: "Aethr on X",
      svg: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20.91 0H24l-7.73 10.54L24 24h-6.89l-5.4-7.9L5.4 24H0l8.17-11.15L0 0h6.97l5 7.36L20.91 0Zm-2.56 21.54h1.91L5.74 2.33H3.72Z"></path></svg>'
    },
    {
      href: "https://discord.gg/XKtbXxG42d",
      label: "Aethr Discord",
      svg: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20.317 4.3698A19.7913 19.7913 0 0 0 16.558 3c-.1963.3427-.4201.8006-.5754 1.1646a18.3345 18.3345 0 0 0-4.964-.7498c-1.6896 0-3.3772.2499-4.9645.7498-.1554-.364-.3791-.8219-.5755-1.1646A19.7363 19.7363 0 0 0 3.677 4.3698C1.2913 7.9778.521 11.488.776 14.9558a19.9117 19.9117 0 0 0 6.032 3.0184c.4875-.6684.9191-1.3766 1.2897-2.1207a12.5903 12.5903 0 0 1-1.9997-.9534c.168-.1235.3319-.2499.49-.3788 3.8329 1.7876 7.9898 1.7876 11.7897 0 .1581.1289.322.2553.49.3788-.6316.3903-1.3037.707-1.9998.9534.3706.7441.8022 1.4523 1.2897 2.1207a19.8887 19.8887 0 0 0 6.033-3.0184c.3938-5.0762-.6762-8.553-.8967-9.741-1.1254-.8208-2.338-1.4519-3.5985-1.7839ZM8.02 11.5375c-.6477 0-1.1767-.5933-1.1767-1.323 0-.7296.521-1.3229 1.1767-1.3229.6643 0 1.1853.6026 1.1767 1.3229 0 .7297-.521 1.323-1.1767 1.323Zm7.96 0c-.6478 0-1.1768-.5933-1.1768-1.323 0-.7296.521-1.3229 1.1768-1.3229.6643 0 1.1853.6026 1.1767 1.3229 0 .7297-.5124 1.323-1.1767 1.323Z"></path></svg>'
    },
    {
      href: "https://github.com/aethrAI",
      label: "Aethr on GitHub",
      svg: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .5C5.65.5.5 5.64.5 12c0 5.09 3.29 9.4 7.86 10.93.58.11.79-.25.79-.56 0-.27-.01-1.17-.02-2.12-3.2.7-3.88-1.37-3.88-1.37-.53-1.34-1.3-1.7-1.3-1.7-1.06-.72.08-.7.08-.7 1.17.08 1.78 1.2 1.78 1.2 1.04 1.78 2.73 1.27 3.4.97.11-.75.41-1.27.74-1.56-2.55-.29-5.23-1.27-5.23-5.65 0-1.25.45-2.27 1.2-3.07-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.17a11.08 11.08 0 0 1 2.9-.39c.99 0 1.99.13 2.92.39 2.2-1.48 3.17-1.17 3.17-1.17.63 1.59.23 2.76.11 3.05.75.8 1.2 1.82 1.2 3.07 0 4.39-2.69 5.35-5.25 5.63.43.36.82 1.07.82 2.17 0 1.57-.02 2.83-.02 3.22 0 .31.21.68.8.56A10.997 10.997 0 0 0 23.5 12C23.5 5.64 18.35.5 12 .5Z"></path></svg>'
    }
  ];

  const init = () => {
    const headerInner = document.querySelector('.md-header__inner');
    if (!headerInner) return;

    const title = headerInner.querySelector('.md-header__title');
    const logoButton = headerInner.querySelector('.md-logo');
    if (title && logoButton && !title.contains(logoButton)) {
      logoButton.classList.add('aethr-logo-inline');
      title.appendChild(logoButton);
    }

    if (!headerInner.querySelector('.aethr-header-links')) {
      const source = headerInner.querySelector('.md-header__source');
      const container = document.createElement('div');
      container.className = 'aethr-header-links';

      ICONS.forEach(({ href, label, svg }) => {
        const anchor = document.createElement('a');
        anchor.href = href;
        anchor.target = '_blank';
        anchor.rel = 'noopener';
        anchor.setAttribute('aria-label', label);
        anchor.innerHTML = svg;
        container.appendChild(anchor);
      });

      headerInner.insertBefore(container, source || null);
    }
  };

  if (document.readyState !== 'loading') {
    init();
  } else {
    document.addEventListener('DOMContentLoaded', init);
  }
})();
