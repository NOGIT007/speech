<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  const WIDTH = 280;
  const HEIGHT = 64;
  const HISTORY_SIZE = 64;

  let canvas: HTMLCanvasElement;
  let samples: number[] = new Array(HISTORY_SIZE).fill(0);
  let animFrame: number;
  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<number>("audio-level", (event) => {
      samples.push(event.payload);
      if (samples.length > HISTORY_SIZE) samples.shift();
    });
    draw();
  });

  onDestroy(() => {
    unlisten?.();
    if (animFrame) cancelAnimationFrame(animFrame);
  });

  function draw() {
    const ctx = canvas?.getContext("2d");
    if (!ctx) {
      animFrame = requestAnimationFrame(draw);
      return;
    }

    ctx.clearRect(0, 0, WIDTH, HEIGHT);

    const midY = HEIGHT / 2;
    const stepX = WIDTH / (HISTORY_SIZE - 1);

    // Draw oscilloscope wave
    ctx.beginPath();
    for (let i = 0; i < samples.length; i++) {
      const level = Math.sqrt(samples[i]); // boost quiet signals
      // Alternate above/below center based on index for oscillator look
      const sign = i % 2 === 0 ? 1 : -1;
      const y = midY + sign * level * (HEIGHT * 0.45);
      const x = i * stepX;

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        // Smooth curve between points
        const prevLevel = Math.sqrt(samples[i - 1]);
        const prevSign = (i - 1) % 2 === 0 ? 1 : -1;
        const prevY = midY + prevSign * prevLevel * (HEIGHT * 0.45);
        const prevX = (i - 1) * stepX;
        const cpX = (prevX + x) / 2;
        ctx.bezierCurveTo(cpX, prevY, cpX, y, x, y);
      }
    }

    ctx.strokeStyle = "rgba(168, 85, 247, 0.9)";
    ctx.lineWidth = 2;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.stroke();

    // Subtle glow
    ctx.strokeStyle = "rgba(168, 85, 247, 0.2)";
    ctx.lineWidth = 6;
    ctx.stroke();

    animFrame = requestAnimationFrame(draw);
  }
</script>

<canvas bind:this={canvas} width={WIDTH} height={HEIGHT} style="width: {WIDTH}px; height: {HEIGHT}px;"></canvas>
