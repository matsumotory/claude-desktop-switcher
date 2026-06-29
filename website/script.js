// docs/script.js

document.addEventListener('DOMContentLoaded', () => {
    // Intersection Observer for fade-in animations
    const observerOptions = {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
    };

    const observer = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    const fadeElements = document.querySelectorAll('.fade-in');
    fadeElements.forEach(el => observer.observe(el));

    // Mobile nav: hamburger toggle (opens the glass panel under 768px)
    const navToggle = document.querySelector('.nav-toggle');
    const navLinks = document.querySelector('.nav-links');
    if (navToggle && navLinks) {
        const setNav = (open) => {
            document.body.classList.toggle('nav-open', open);
            navToggle.setAttribute('aria-expanded', open ? 'true' : 'false');
        };
        navToggle.addEventListener('click', (e) => {
            e.stopPropagation();
            setNav(!document.body.classList.contains('nav-open'));
        });
        navLinks.querySelectorAll('a').forEach((a) => {
            a.addEventListener('click', () => setNav(false));
        });
        document.addEventListener('click', (e) => {
            if (document.body.classList.contains('nav-open') &&
                !navLinks.contains(e.target) && !navToggle.contains(e.target)) {
                setNav(false);
            }
        });
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') setNav(false);
        });
        window.addEventListener('resize', () => {
            if (window.innerWidth > 768) setNav(false);
        });
    }
});
