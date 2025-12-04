import type { EngineCmdEnvelope } from './cmds';
import {
  vulframDispose,
  vulframGetBenchmarks,
  vulframInit,
  vulframReceiveQueue,
  vulframResetBenchmarks,
  vulframSetBenchmark,
  VulframResult,
  vulframSendQueue,
  vulframTick,
} from './index';

// Enable benchmark tracking
vulframSetBenchmark(true);

// Initialize engine
console.log('Initializing Vulfram Engine...');
const initResult = vulframInit();
if (initResult !== VulframResult.Success) {
  console.error('Failed to initialize engine:', initResult);
  process.exit(1);
}
console.log('Engine initialized successfully!');

// Track command IDs
let nextCmdId = 1;
const windowIds: number[] = [];

// Create two windows
console.log('\nCreating windows...');
const createCommands: EngineCmdEnvelope[] = [
  {
    id: nextCmdId++,
    type: 'cmd-window-create',
    content: {
      title: 'Window 1 - Press ESC to close all',
      size: [800, 600],
      position: [100, 100],
      borderless: false,
      resizable: true,
      initialState: 'windowed',
    },
  },
  {
    id: nextCmdId++,
    type: 'cmd-window-create',
    content: {
      title: 'Window 2 - Press ESC to close all',
      size: [800, 600],
      position: [950, 100],
      borderless: false,
      resizable: true,
      initialState: 'windowed',
    },
  },
];

vulframSendQueue(createCommands);

// Main loop
let running = true;
let lastTime = Date.now();

console.log('\nStarting main loop...');
console.log('Press ESC in any window to close all windows and exit.\n');

function processEvents() {
  const [events, result] = vulframReceiveQueue();

  if (result !== VulframResult.Success) {
    console.error('Failed to receive events:', result);
    return;
  }

  for (const event of events) {
    // Handle command results
    if (event.type === 'window-create') {
      const content = event.content;
      if (content.success) {
        windowIds.push(content.content);
        console.log(`✓ Window created with ID: ${content.content}`);
      } else {
        console.error(`✗ Failed to create window: ${content.message}`);
      }
    }

    // Handle window events
    if (event.type === 'window') {
      const windowEvent = event.content;

      if (windowEvent.event === 'on-close-request') {
        console.log(`Window ${windowEvent.data.windowId} close requested`);
        running = false;
      }

      if (windowEvent.event === 'on-destroy') {
        console.log(`Window ${windowEvent.data.windowId} destroyed`);
      }

      if (windowEvent.event === 'on-resize') {
        console.log(
          `Window ${windowEvent.data.windowId} resized to ${windowEvent.data.width}x${windowEvent.data.height}`,
        );
      }

      if (windowEvent.event === 'on-focus') {
        console.log(
          `Window ${windowEvent.data.windowId} focus: ${windowEvent.data.focused}`,
        );
      }
    }

    // Handle keyboard events
    if (event.type === 'keyboard') {
      const keyboardEvent = event.content;

      if (keyboardEvent.event === 'on-input') {
        const keyData = keyboardEvent.data;

        // Check for ESC key press
        if (keyData.keyCode === 'escape' && keyData.state === 'pressed') {
          console.log('\n🔴 ESC pressed! Closing all windows...\n');

          // Send close commands for all windows
          const closeCommands: EngineCmdEnvelope[] = windowIds.map(
            (windowId) => ({
              id: nextCmdId++,
              type: 'cmd-window-close',
              content: { windowId },
            }),
          );

          vulframSendQueue(closeCommands);
          running = false;
        }

        // Log other key presses
        if (keyData.state === 'pressed' && !keyData.repeat) {
          console.log(
            `Key pressed: ${keyData.keyCode} in window ${keyData.windowId}`,
          );
        }
      }
    }

    // Handle pointer events
    if (event.type === 'pointer') {
      const pointerEvent = event.content;

      if (pointerEvent.event === 'on-button') {
        const buttonData = pointerEvent.data;
        if (buttonData.state === 'pressed') {
          console.log(
            `Mouse button ${JSON.stringify(buttonData.button)} pressed at [${buttonData.position[0].toFixed(1)}, ${buttonData.position[1].toFixed(1)}] in window ${buttonData.windowId}`,
          );
        }
      }
    }

    // Handle system events
    if (event.type === 'system') {
      const systemEvent = event.content;

      if (systemEvent.event === 'on-exit') {
        console.log('System exit event received');
        running = false;
      }
    }
  }
}

// Main loop
const loopInterval = setInterval(() => {
  if (!running) {
    clearInterval(loopInterval);

    // Print benchmark results
    console.log('\n📊 Benchmark Results:');
    console.log('='.repeat(80));
    const benchmarks = vulframGetBenchmarks();

    for (const bench of benchmarks) {
      console.log(`\n${bench.name}:`);
      console.log(`  Calls:       ${bench.calls.toLocaleString()}`);
      console.log(`  Total:       ${bench.totalMs.toFixed(3)} ms`);
      console.log(
        `  Avg:         ${bench.avgUs.toFixed(3)} µs (${bench.avgNs.toFixed(0)} ns)`,
      );
      console.log(`  Min:         ${bench.minUs.toFixed(3)} µs`);
      console.log(`  Max:         ${bench.maxUs.toFixed(3)} µs`);
      console.log(`  Last:        ${bench.lastUs.toFixed(3)} µs`);
    }
    console.log('\n' + '='.repeat(80));

    // Cleanup
    console.log('\nDisposing engine...');
    const disposeResult = vulframDispose();
    if (disposeResult === VulframResult.Success) {
      console.log('Engine disposed successfully!');
    } else {
      console.error('Failed to dispose engine:', disposeResult);
    }

    process.exit(0);
  }

  // Calculate delta time
  const currentTime = Date.now();
  const deltaTime = currentTime - lastTime;
  lastTime = currentTime;

  // Tick engine
  vulframTick(currentTime, deltaTime);

  // Process events
  processEvents();
}, 16); // ~60 FPS

// Handle process termination
process.on('SIGINT', () => {
  console.log('\n\nReceived SIGINT, shutting down...');
  running = false;
});

process.on('SIGTERM', () => {
  console.log('\n\nReceived SIGTERM, shutting down...');
  running = false;
});
