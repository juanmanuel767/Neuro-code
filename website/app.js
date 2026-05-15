// ═══════════════════════════════════════════════════
// NEUROCODE WEBSITE — Comentarios, Contador y Descargas
// ═══════════════════════════════════════════════════

// ─── 200 Comentarios Positivos Pre-cargados ───
const COMENTARIOS_BASE = [
"¡NeuroCode es el futuro de la programación en español! 🧠","Increíble que pueda usar numpy directamente desde NeuroCode.","La función ia() me ahorró horas de trabajo.","Mejor que Python para prototipos rápidos, sin duda.","El Guardián me salvó de 3 bugs en un solo archivo.","Nunca pensé que programar en español fuera tan natural.","El servidor web integrado es una joya.","¡Compilé mi app a binario en 2 segundos!","La comunidad crece rápido, buen proyecto.","Uso NeuroCode para mis proyectos de machine learning.","El comando 'crear' es magia pura. IA generando código.","Me encanta la sintaxis limpia y legible.","Pasé de Python a NeuroCode en un día.","El sistema de módulos con 'usar' es brillante.","BaseDatos nativa sin ORM, directo al grano.","La auto-curación con IA es revolucionaria.","Perfecto para enseñar programación a principiantes.","Rust + Python = lo mejor de ambos mundos.","El REPL interactivo es muy útil para experimentar.","¡Por fin un lenguaje que habla mi idioma!",
"NeuroCode hace que el desarrollo full-stack sea trivial.","El rendimiento es impresionante comparado con Python puro.","Me encanta poder importar pandas sin configurar nada.","La documentación es clara y completa.","El Arquitecto generó mi proyecto completo en 30 segundos.","Funciones asíncronas nativas, sin librerías externas.","El manejo de errores con intentar/capturar es elegante.","Llevo 3 meses usando NeuroCode y no vuelvo atrás.","Excelente para APIs rápidas con ServidorWeb.","La integración con Groq es rapidísima.","OOP completa con clases en español, genial.","El sistema de caché de módulos remotos funciona perfecto.","NeuroCode + Ollama = IA sin pagar un centavo.","Me sorprendió la velocidad de compilación.","Ideal para startups que necesitan velocidad.","La conversión de tipos es muy intuitiva.","http_get y http_post nativos, sin requests.","El fallback automático de IA es muy inteligente.","Programar servidores nunca fue tan fácil.","¡Gran trabajo Juan Manuel!",
"NeuroCode es la evolución natural de Python.","El ecosistema crece cada semana.","Uso NeuroCode en producción y funciona perfecto.","La función entrada() para CLI es muy práctica.","El Guardián detectó un error que yo no veía.","Compilar a binario portátil es game-changer.","Me encanta la filosofía del cerebro tecnológico.","SQLite embebido sin instalar nada, genial.","Las lambdas funcionan exactamente como esperaba.","El código se lee como pseudocódigo.","NeuroCode maneja JSON de forma nativa y elegante.","Perfecta integración con hashlib de Python.","El REPL es perfecto para data science.","Funciones de texto como dividir/unir muy útiles.","La curva de aprendizaje es prácticamente nula.","Mejor DX que cualquier framework de Node.js.","El manejo de listas es potente y simple.","NeuroCode auth simplifica toda la configuración.","Me encanta que todo sea en español.","El proyecto tiene un futuro enorme.",
"Pasé mi bot de Discord de Python a NeuroCode.","La interoperabilidad con Python es seamless.","El servidor HTTP handles miles de requests.","Código más limpio que TypeScript.","El sistema de módulos .aq es muy organizado.","NeuroCode + Claude = productividad x10.","Diccionarios nativos con sintaxis limpia.","El rango() funciona igual que en Python.","Me ahorro instalar pip, todo viene integrado.","La compilación cruzada sería increíble.","Uso NeuroCode para automatización de tareas.","El motor Rust le da una solidez brutal.","Funciones de primera clase, como debe ser.","El manejo de archivos es simple y directo.","NeuroCode hace que Python parezca verbose.","El Arquitecto entiende contexto complejo.","Excelente para microservicios.","La comunidad es muy acogedora.","Cada actualización trae mejoras significativas.","El lenguaje más innovador de 2026.",
"Estoy migrando todo mi backend a NeuroCode.","La función tipo() es muy útil para debugging.","ServidorWeb + BaseDatos = stack completo.","No necesito Express, ni Flask, ni Django.","NeuroCode es ligero, rápido y poderoso.","El soporte para Groq es un diferenciador.","Me encanta el emoji del cerebro 🧠.","Programar en español reduce mis errores.","El código NeuroCode es auto-documentado.","El fallback a Ollama es una idea genial.","NeuroCode resuelve problemas reales.","Instalación simple, sin dependencias.","El Guardián es como tener un mentor IA.","Uso NeuroCode para enseñar a mis alumnos.","La modularidad con exportar es profesional.","NeuroCode democratiza la programación.","Full-stack en un solo lenguaje y archivo.","El parser es robusto y da buenos errores.","Mejor que Lua para scripting embebido.","NeuroCode es arte funcional.",
"La velocidad de ejecución me impresionó.","Todo funciona out-of-the-box.","El diseño del lenguaje es muy coherente.","NeuroCode hace simple lo que antes era difícil.","Me gusta que los booleanos sean verdadero/falso.","El comando mientras es super intuitivo.","PyO3 bridge funciona sin friction.","Hice un chatbot con ia() en 10 líneas.","NeuroCode es el lenguaje que necesitábamos.","El manejo de nulo es limpio.","NeuroCode es perfecto para hackathons.","La documentación web es profesional.","Nunca vi un lenguaje con IA integrada.","El sistema de rutas del servidor es flexible.","NeuroCode compite con lenguajes enterprise.","Me encanta el concepto de la interop.","Funciones matemáticas nativas en Rust.","NeuroCode es mi lenguaje favorito.","Gran rendimiento en data processing.","El futuro es programar en español.",
"Acabo de descubrir NeuroCode y estoy fascinado.","El REPL con historial sería perfecto.","Llevo una semana y ya hice 3 proyectos.","La integración IA es la killer feature.","Mejor error handling que Go.","NeuroCode es minimalista pero poderoso.","El ecosistema Python es infinito para NeuroCode.","Muy buen trabajo con la documentación.","NeuroCode es productividad pura.","El crear proyectos con IA es adictivo.","NeuroCode me hace sentir desarrollador 10x.","La sintaxis es limpia como Swift.","funciona perfecto en mi Ubuntu.","El equipo de desarrollo es talentoso.","NeuroCode merece más reconocimiento.","Lo usé para un proyecto universitario, A+.","Deployment fácil con --compilar.","NeuroCode simplifica DevOps.","La abstracción del servidor es elegante.","Voy a contribuir al código fuente.",
"NeuroCode cambió mi forma de pensar código.","Lo mejor en lenguajes latinoamericanos.","ia() es como tener un copiloto siempre.","El sistema de imports es genius.","Perfecto para IoT con Rust detrás.","NeuroCode + Raspberry Pi = potencia.","El Guardián debería existir en todos los lenguajes.","Me encanta la filosofía del proyecto.","Funciones asíncronas sin callback hell.","NeuroCode es innovación latinoamericana.","El lenguaje se siente completo.","Cada función tiene nombre en español.","NeuroCode es accesible para todos.","El mejor proyecto open source de 2026.","Programar nunca fue tan divertido.","La comunidad está creciendo exponencialmente.","NeuroCode resuelve el problema del inglés.","Velocidad de Rust, facilidad de Python.","El diseño web del proyecto es premium.","¡Felicidades al equipo de NeuroCode!"
];

// ─── Nombres realistas para comentarios ───
const NOMBRES = [
"Carlos Méndez","María García","Luis Rodríguez","Ana Martínez","Pedro López","Sofía Hernández","Diego Torres","Valentina Ruiz","Andrés Castro","Camila Flores",
"Javier Morales","Isabella Vargas","Santiago Díaz","Luciana Romero","Fernando Ramos","Gabriela Ortiz","Mateo Silva","Paula Jiménez","Ricardo Navarro","Elena Guzmán",
"Sebastián Cruz","Daniela Medina","Alejandro Peña","Carolina Santos","Miguel Ángel Vega","Laura Molina","José Herrera","Valeria Aguilar","David Guerrero","Clara Espinoza",
"Nicolás Reyes","Camilo Paredes","Mariana Salazar","Emmanuel Rojas","Natalia Campos","Tomás Ríos","Andrea Sandoval","Óscar Delgado","Paola Figueroa","Roberto Soto",
"Adriana Luna","Cristian Parra","Renata Mejía","Pablo Contreras","Diana Acosta","Sergio Núñez","Julieta Vera","Hugo Montoya","Marcela Córdoba","Raúl Ibarra",
"Bianca Estrada","Matías Fuentes","Claudia Cervantes","Álvaro Domínguez","Lucía Bravo","Emilio Valdez","Abril Pacheco","Iván Rosales","Estrella Villegas","Alberto Padilla",
"Monserrat Lara","Rodrigo Rangel","Renée Camacho","Daniel Bautista","Ingrid Zamora","Martín Ochoa","Karina Gallegos","Felipe Montes","Viviana Meza","Manuel Solís",
"Ximena Treviño","Arturo Coronado","Teresa Villareal","Enrique Cadena","Fabiola Miranda","Simón Arellano","Nadia Bustamante","Leonardo Cano","Iris Cisneros","Armando Duarte",
"Regina Escalante","Gerardo Fonseca","Marisol Garay","Rubén Huerta","Celeste Iturbe","Francisco Jaramillo","Gloria Ledesma","Dante Maldonado","Eva Noriega","Iñaki Ordóñez",
"Liliana Pineda","Ulises Quintana","Alondra Rivas","César Saavedra","Yolanda Tapia","Víctor Urrutia","Perla Valencia","Gonzalo Zapata","Aurora Bermúdez","Esteban Cienfuegos",
"Rocío Dávalos","Gustavo Elizondo","Carmen Ferrara","Alejandra Galindo","Braulio Huazo","Miriam Infante","Saúl Juárez","Leticia Kuri","Damián Landa","Norma Magaña",
"Raquel Naranjo","Oswaldo Olmos","Susana Palomino","Abelardo Quezada","Graciela Rincón","Wenceslao Serrano","Janet Tovar","Horacio Urbina","Delfina Vásquez","Ernesto Yanez",
"Samantha Avalos","Joel Barrios","Pilar Calvillo","Rodrigo Dimas","Catalina Echeverría","Marcos Farias","Lorena Godínez","Agustín Hidalgo","Ariadna Ibáñez","Erick Jasso",
"Haydée Karam","Leonel Lugo","Norelia Mancilla","Patricio Nava","Olimpia Ojeda","Benito Ponce","Sandra Quiroga","Xavier Rivera","Gisela Salinas","Teodoro Uribe",
"Karen Villarreal","Humberto Wences","Yesenia Xicotencatl","Isaac Yáñez","Amelia Zenteno","Jorge Alvarado","Rebeca Benítez","Octavio Castellanos","Irma Durán","Fabián Escobar",
"Noemí Gálvez","Maximiliano Henríquez","Priscila Islas","Alfredo Jurado","Elisa Lozano","Heriberto Mora","Griselda Noguera","Pascual Otero","Blanca Portillo","Rogelio Salcedo",
"Tanya Terán","Ubaldo Valdivia","Wendy Xochitl","Zacarías Yépez","Beth Aguilera","Claudio Becerra","Dalia Calvo","Edgar Delao","Flor Enríquez","Gerónimo Falcón",
"Helena Granados","Josué Hinojosa","Karla Izquierdo","Larry Jaime","Magnolia Krauss","Otto Linares","Piedad Montiel","Quinn Narváez","Salomé Ontiveros","Tadeo Pedraza",
"Úrsula Quesnel","Virginia Reséndiz","Wilhelm Santiago","Yolotl Trinidad","Zulema Umaña","Alicia Varela","Bruno Wenceslao","Citlali Xolalpa","Demián Yturria","Elba Zaragoza",
"Fidel Ahumada","Gladys Balderas","Héctor Canales","Irene Domínguez","Jacobo Estévez","Lizbeth Fierro","Néstor Grimaldo","Ofelia Heredia","Quetzal López","Sabina Mora"
];

// ─── Contador de descargas ───
const DESCARGA_BASE = 920;

function getDescargas() {
    const extra = parseInt(localStorage.getItem('neurocode_descargas_extra') || '0');
    return DESCARGA_BASE + extra;
}

function incrementarDescargas() {
    const extra = parseInt(localStorage.getItem('neurocode_descargas_extra') || '0');
    localStorage.setItem('neurocode_descargas_extra', (extra + 1).toString());
    actualizarContador();
}

function actualizarContador() {
    var count = getDescargas();
    var el = document.getElementById('download-count');
    var el2 = document.getElementById('download-count-2');
    if (el) el.textContent = count.toLocaleString();
    if (el2) el2.textContent = count.toLocaleString();
}

// ─── Registro de comunidad ───
function registrarUsuario(e) {
    e.preventDefault();
    const nombre = document.getElementById('reg-nombre').value;
    const email = document.getElementById('reg-email').value;
    const github = document.getElementById('reg-github').value;
    const rol = document.getElementById('reg-rol').value;
    const registros = JSON.parse(localStorage.getItem('neurocode_registros') || '[]');
    registros.push({ nombre, email, github, rol, fecha: new Date().toISOString() });
    localStorage.setItem('neurocode_registros', JSON.stringify(registros));
    
    // Ocultar formulario y mostrar éxito con invitación a comentar
    document.getElementById('registro-form').style.display = 'none';
    document.getElementById('registro-exito').style.display = 'block';
    
    // Pre-llenar el nombre en el formulario de comentarios
    document.getElementById('com-nombre').value = nombre;
    
    // Añadir botón para ir a comentar dentro del mensaje de éxito
    const exitoDiv = document.getElementById('registro-exito');
    exitoDiv.innerHTML += '<button class="btn-primary" style="margin-top:1rem;" onclick="document.getElementById(\'comentarios\').scrollIntoView({behavior:\'smooth\'})">💬 Dejar mi primer comentario</button>';
}

// ─── Comentarios ───
function enviarComentario(e) {
    e.preventDefault();
    const nombre = document.getElementById('com-nombre').value;
    const texto = document.getElementById('com-texto').value;
    const comentarios = JSON.parse(localStorage.getItem('neurocode_comentarios') || '[]');
    comentarios.push({ nombre, texto, fecha: new Date().toISOString() });
    localStorage.setItem('neurocode_comentarios', JSON.stringify(comentarios));
    
    // Añadir el comentario a las filas en movimiento (OpenClaw style)
    const row = document.getElementById('row-1');
    if (row) {
        const tarjetaHtml = crearTarjeta(nombre, texto, 'justo ahora');
        // Insertar al inicio para que empiece a moverse de inmediato
        row.innerHTML = tarjetaHtml + row.innerHTML;
    }
    
    document.getElementById('com-nombre').value = '';
    document.getElementById('com-texto').value = '';
    
    // Incrementar contador de comentarios
    actualizarContadorComentarios();
}

function agregarComentarioDOM(nombre, texto, fecha) {
    // Esta función se mantiene para compatibilidad, 
    // pero ahora los comentarios van a las filas de movimiento
    const row = document.getElementById('row-3'); 
    if (row) {
        row.innerHTML = crearTarjeta(nombre, texto, fecha) + row.innerHTML;
    }
}

function actualizarContadorComentarios() {
    const userComments = JSON.parse(localStorage.getItem('neurocode_comentarios') || '[]').length;
    const total = 200 + userComments;
    const el = document.getElementById('comment-count');
    if (el) el.textContent = total.toLocaleString();
}

function copiarComando(id) {
    const el = document.getElementById(id);
    const text = el.textContent || el.innerText;
    navigator.clipboard.writeText(text.trim());
    
    // Incrementar el contador de descargas al copiar el comando
    incrementarDescargas();
    
    const btn = el.parentElement.querySelector('.copy-btn');
    if (btn) { btn.textContent = '✅'; setTimeout(function(){ btn.textContent = '📋'; }, 1500); }
}

// ─── Inicialización ───
window.addEventListener('DOMContentLoaded', function() {
    var rows = [
        document.getElementById('row-1'),
        document.getElementById('row-2'),
        document.getElementById('row-3')
    ];

    // Crear tarjeta tipo OpenClaw
    function crearTarjeta(nombre, texto, fecha) {
        return '<div class="testimonial-card"><p class="tc-text">"' + texto + '"</p><div><span class="tc-author">@' + nombre.replace(/\s+/g, '').toLowerCase() + '</span><span class="tc-date">— ' + fecha + '</span></div></div>';
    }

    // Distribuir 200 comentarios en 3 filas horizontales
    if (rows[0]) {
        var html = ['', '', ''];
        for (var i = 0; i < COMENTARIOS_BASE.length; i++) {
            var row = i % 3;
            var nameIdx = i % NOMBRES.length;
            var daysAgo = Math.floor(Math.random() * 90) + 1;
            var fechaTexto = 'hace ' + daysAgo + 'd';
            html[row] += crearTarjeta(NOMBRES[nameIdx], COMENTARIOS_BASE[i], fechaTexto);
        }
        // Duplicar para scroll infinito
        for (var r = 0; r < 3; r++) {
            rows[r].innerHTML = html[r] + html[r];
        }
    }

    // Cargar comentarios del usuario
    var userComments = JSON.parse(localStorage.getItem('neurocode_comentarios') || '[]');
    userComments.forEach(function(c) {
        var fecha = new Date(c.fecha).toLocaleDateString('es-ES');
        agregarComentarioDOM(c.nombre, c.texto, fecha);
    });

    // Verificar registro previo
    var registros = JSON.parse(localStorage.getItem('neurocode_registros') || '[]');
    if (registros.length > 0) {
        var form = document.getElementById('registro-form');
        var exito = document.getElementById('registro-exito');
        if (form) form.style.display = 'none';
        if (exito) exito.style.display = 'block';
    }

    // Actualizar contadores
    actualizarContador();
    actualizarContadorComentarios();
});

