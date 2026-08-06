import sharp from 'sharp';
import fs from 'fs';
import path from 'path';

const SVG_PATH = path.resolve('icon.svg');
const ICONS_DIR = path.resolve('src-tauri/icons');
const IOS_DIR = path.join(ICONS_DIR, 'ios');
const ANDROID_DIR = path.join(ICONS_DIR, 'android');

const svgContent = fs.readFileSync(SVG_PATH, 'utf-8');

// Render SVG: use density to get 2x-4x oversampled intermediate for crisp results
async function renderSVG(size) {
  const density = Math.max(300, Math.ceil(72 * size * 3 / 1024));
  return sharp(Buffer.from(svgContent), { density })
    .resize(size, size, { fit: 'cover', background: { r: 10, g: 10, b: 15, alpha: 0 } })
    .png()
    .toBuffer();
}

// ICO generation: header + directory entries + PNG data
function createIco(pngBuffers, sizes) {
  const numImages = pngBuffers.length;
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);     // reserved
  header.writeUInt16LE(1, 2);     // type: ICO
  header.writeUInt16LE(numImages, 4); // count

  let offset = 6 + numImages * 16;
  const dirEntries = [];
  const imageData = [];

  for (let i = 0; i < numImages; i++) {
    const png = pngBuffers[i];
    const size = sizes[i];
    const w = size >= 256 ? 0 : size;
    const h = size >= 256 ? 0 : size;
    const entry = Buffer.alloc(16);
    entry.writeUInt8(w, 0);
    entry.writeUInt8(h, 1);
    entry.writeUInt8(0, 2);  // colors
    entry.writeUInt8(0, 3);  // reserved
    entry.writeUInt16LE(1, 4);  // planes
    entry.writeUInt16LE(32, 6); // bpp
    entry.writeUInt32LE(png.length, 8); // size
    entry.writeUInt32LE(offset, 12); // offset
    dirEntries.push(entry);
    imageData.push(png);
    offset += png.length;
  }

  return Buffer.concat([header, ...dirEntries, ...imageData]);
}

// ICNS generation
function createIcns(pngBuffers, types, sizes) {
  const entries = [];
  for (let i = 0; i < pngBuffers.length; i++) {
    const png = pngBuffers[i];
    const iconType = types[i];
    const entrySize = 8 + png.length;
    const entry = Buffer.alloc(8);
    entry.write(iconType, 0, 4, 'ascii');
    entry.writeUInt32BE(entrySize, 4);
    entries.push(Buffer.concat([entry, png]));
  }

  const totalEntries = Buffer.concat(entries);
  const header = Buffer.alloc(8);
  header.write('icns', 0, 4, 'ascii');
  header.writeUInt32BE(8 + totalEntries.length, 4);
  return Buffer.concat([header, totalEntries]);
}

async function main() {
  // ============================================
  // 1. Bundle PNGs (root icons directory)
  // ============================================
  console.log('Generating bundle PNG icons...');
  const pngSizes = [32, 64, 128, 256, 512, 1024];
  const pngResults = {};
  for (const size of pngSizes) {
    pngResults[size] = await renderSVG(size);
    const filename = size === 1024 ? 'icon.png' : `${size}x${size}.png`;
    fs.writeFileSync(path.join(ICONS_DIR, filename), pngResults[size]);
    console.log(`  Generated ${filename} (${size}x${size})`);
  }
  // 128x128@2x = 256x256
  fs.writeFileSync(path.join(ICONS_DIR, '128x128@2x.png'), pngResults[256]);
  console.log('  Generated 128x128@2x.png (256x256)');

  // ============================================
  // 2. ICO file (Windows)
  // ============================================
  console.log('Generating ICO file...');
  const icoSizes = [32, 24, 16, 48, 64, 128, 256];
  const icoPngs = [];
  for (const size of icoSizes) {
    icoPngs.push(await renderSVG(size));
  }
  const ico = createIco(icoPngs, icoSizes);
  fs.writeFileSync(path.join(ICONS_DIR, 'icon.ico'), ico);
  console.log(`  Generated icon.ico (${icoSizes.join(', ')}) - ${ico.length} bytes`);

  // ============================================
  // 3. ICNS file (macOS)
  // ============================================
  console.log('Generating ICNS file...');
  // ic07=128, ic08=256, ic09=512, ic10=1024
  const icnsPngs = [pngResults[128], pngResults[256], pngResults[512], pngResults[1024]];
  const icnsTypes = ['ic07', 'ic08', 'ic09', 'ic10'];
  const icnsSizes = [128, 256, 512, 1024];
  const icns = createIcns(icnsPngs, icnsTypes, icnsSizes);
  fs.writeFileSync(path.join(ICONS_DIR, 'icon.icns'), icns);
  console.log(`  Generated icon.icns - ${icns.length} bytes`);

  // ============================================
  // 4. Windows Store logo tiles
  // ============================================
  console.log('Generating Windows Store logo tiles...');
  const tiles = [
    { name: 'StoreLogo.png', size: 50 },
    { name: 'Square30x30Logo.png', size: 30 },
    { name: 'Square44x44Logo.png', size: 44 },
    { name: 'Square71x71Logo.png', size: 71 },
    { name: 'Square89x89Logo.png', size: 89 },
    { name: 'Square107x107Logo.png', size: 107 },
    { name: 'Square142x142Logo.png', size: 142 },
    { name: 'Square150x150Logo.png', size: 150 },
    { name: 'Square284x284Logo.png', size: 284 },
    { name: 'Square310x310Logo.png', size: 310 },
  ];
  for (const tile of tiles) {
    const buf = await renderSVG(tile.size);
    fs.writeFileSync(path.join(ICONS_DIR, tile.name), buf);
    console.log(`  Generated ${tile.name} (${tile.size}x${tile.size})`);
  }

  // ============================================
  // 5. iOS icons
  // ============================================
  console.log('Generating iOS icons...');
  fs.mkdirSync(IOS_DIR, { recursive: true });
  const iosIcons = [
    { name: 'AppIcon-20x20@1x.png', size: 20 },
    { name: 'AppIcon-20x20@2x.png', size: 40 },
    { name: 'AppIcon-20x20@2x-1.png', size: 40 },
    { name: 'AppIcon-20x20@3x.png', size: 60 },
    { name: 'AppIcon-29x29@1x.png', size: 29 },
    { name: 'AppIcon-29x29@2x.png', size: 58 },
    { name: 'AppIcon-29x29@2x-1.png', size: 58 },
    { name: 'AppIcon-29x29@3x.png', size: 87 },
    { name: 'AppIcon-40x40@1x.png', size: 40 },
    { name: 'AppIcon-40x40@2x.png', size: 80 },
    { name: 'AppIcon-40x40@2x-1.png', size: 80 },
    { name: 'AppIcon-40x40@3x.png', size: 120 },
    { name: 'AppIcon-512@2x.png', size: 1024 },
    { name: 'AppIcon-60x60@2x.png', size: 120 },
    { name: 'AppIcon-60x60@3x.png', size: 180 },
    { name: 'AppIcon-76x76@1x.png', size: 76 },
    { name: 'AppIcon-76x76@2x.png', size: 152 },
    { name: 'AppIcon-83.5x83.5@2x.png', size: 167 },
  ];
  for (const icon of iosIcons) {
    const buf = await renderSVG(icon.size);
    fs.writeFileSync(path.join(IOS_DIR, icon.name), buf);
    console.log(`  Generated ios/${icon.name} (${icon.size}x${icon.size})`);
  }

  // ============================================
  // 6. Android icons
  // ============================================
  console.log('Generating Android icons...');
  const androidDensities = [
    { density: 'mdpi', scale: 1 },
    { density: 'hdpi', scale: 1.5 },
    { density: 'xhdpi', scale: 2 },
    { density: 'xxhdpi', scale: 3 },
    { density: 'xxxhdpi', scale: 4 },
  ];
  const androidBaseSize = 48; // mdpi base

  // Generate launcher icons for each density
  for (const { density, scale } of androidDensities) {
    const dir = path.join(ANDROID_DIR, `mipmap-${density}`);
    fs.mkdirSync(dir, { recursive: true });
    
    const size = Math.round(androidBaseSize * scale);
    const foregroundSize = Math.round(androidBaseSize * scale);
    
    // Regular launcher icon
    const buf = await renderSVG(size);
    fs.writeFileSync(path.join(dir, 'ic_launcher.png'), buf);
    
    // Foreground (same as regular for simplicity)
    const fgBuf = await renderSVG(foregroundSize);
    fs.writeFileSync(path.join(dir, 'ic_launcher_foreground.png'), fgBuf);
    
    // Round icon (same as regular)
    fs.writeFileSync(path.join(dir, 'ic_launcher_round.png'), buf);
    
    console.log(`  Generated android/mipmap-${density}/ (${size}px)`);
  }

  // Generate Android adaptive icon XML
  const anydpiDir = path.join(ANDROID_DIR, 'mipmap-anydpi-v26');
  fs.mkdirSync(anydpiDir, { recursive: true });

  const launcherXml = `<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <background android:drawable="@color/ic_launcher_background"/>
  <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>`;
  fs.writeFileSync(path.join(anydpiDir, 'ic_launcher.xml'), launcherXml);

  // Android values with background color
  const valuesDir = path.join(ANDROID_DIR, 'values');
  fs.mkdirSync(valuesDir, { recursive: true });
  const colorsXml = `<?xml version="1.0" encoding="utf-8"?>
<resources>
  <color name="ic_launcher_background">#0A0A0F</color>
</resources>`;
  fs.writeFileSync(path.join(valuesDir, 'ic_launcher_background.xml'), colorsXml);

  console.log('  Generated android adaptive icon XML files');

  // ============================================
  // Summary
  // ============================================
  console.log('\n=== Icon Generation Complete ===');
  console.log('All 52+ icon assets have been regenerated from the custom SVG logo.');
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
