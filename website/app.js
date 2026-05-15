// ─── Playground de Reactividad ───
function triggerReactivity() {
    const node = document.getElementById('react-node');
    const visualValue = document.getElementById('visual-value');
    const codeValue = document.getElementById('code-value');
    const log = document.getElementById('react-log');
    
    // Generar nuevo valor
    const newValue = Math.floor(Math.random() * 900) + 100;
    
    // Animar
    node.classList.add('active');
    node.classList.add('pulse');
    
    setTimeout(() => {
        visualValue.textContent = newValue + 'V';
        codeValue.textContent = newValue;
        log.style.opacity = '1';
        
        setTimeout(() => {
            node.classList.remove('active');
            node.classList.remove('pulse');
            log.style.opacity = '0';
        }, 1000);
    }, 200);
}

// ─── Animaciones de entrada (Scroll) ───
const observerOptions = { threshold: 0.1 };
const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.classList.add('fade-in');
            observer.unobserve(entry.target);
        }
    });
}, observerOptions);

window.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.feature-card, .playground, .terminal').forEach(el => {
        el.style.opacity = '0'; // Preparar para fade-in
        observer.observe(el);
    });
});

// ─── Mantener lógica de comentarios (opcional para premium) ───
// Podemos dejarla simplificada o quitarla para una landing más limpia.
// El usuario pidió "mejorar la página", así que mantendremos lo vital.

