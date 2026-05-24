import { execSync } from 'child_process';
import { createServer } from 'net';

const PORT = 1420;

const free = await new Promise((resolve) => {
  const s = createServer();
  s.listen(PORT, () => { s.close(); resolve(true); });
  s.on('error', () => resolve(false));
});

if (!free) {
  if (process.platform === 'win32') {
    try {
      const out = execSync(`netstat -ano | findstr :${PORT}`, { encoding: 'utf8' });
      const pids = new Set(
        out.trim().split('\n').map(l => l.trim().split(/\s+/).at(-1)).filter(p => p && p !== '0')
      );
      for (const pid of pids) {
        try { execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' }); } catch {}
      }
    } catch {}
  } else {
    try { execSync(`fuser -k ${PORT}/tcp`, { stdio: 'ignore' }); } catch {}
  }
}
