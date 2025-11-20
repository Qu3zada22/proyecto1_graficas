🎮 Proyecto 1 — Raycasting FPS

Un juego en primera persona con mecánicas de sigilo, coleccionables, enemigos con IA básica y un sistema de raycasting hecho desde cero en Rust, utilizando la librería Raylib.

✨ Abajo encontrarás imágenes, instrucciones y un espacio para el video del gameplay.

📂 Estructura del Proyecto
raycasting_graficas/
├── project/
│   ├── assets/             # Texturas, sprites y sonidos
│   ├── src/                # Código fuente
│   │   ├── audio.rs        # Manejo del audio y efectos
│   │   ├── caster.rs       # Núcleo del raycasting
│   │   ├── collectable.rs  # Lógica de objetos recogibles
│   │   ├── enemy.rs        # IA y comportamiento de enemigos
│   │   ├── framebuffer.rs  # Render de la pantalla
│   │   ├── main.rs         # Punto de entrada del juego
│   │   ├── maze.rs         # Generación del laberinto
│   │   ├── player.rs       # Movimiento y acciones del jugador
│   │   └── textures.rs     # Carga y administración de texturas
│   ├── maze.txt            # Mapa del nivel fácil
│   ├── maze_hard.txt       # Mapa del nivel difícil
│   └── Cargo.toml          # Configuración y dependencias
└── README.md               # Este archivo


1. Clonar el repositorio
git clone https://github.com/Qu3zada22/proyecto1_graficas.git

2. Ejecutar el juego
cd project
cargo run --release

Controles
En el menú principal

1 → Nivel fácil

2 → Nivel difícil

ESC → Salir

Durante el juego

W / ↑ → Avanzar

S / ↓ → Retroceder

A / ← → Girar a la izquierda

D / → → Girar a la derecha

Click Izquierdo → Avanzar

Click Derecho → Retroceder

M → Cambiar entre vista 2D / 3D

TAB → Volver al menú

ESC → Salir

🎥 Video de Gameplay

link: 
