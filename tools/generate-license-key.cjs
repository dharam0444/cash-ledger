const crypto = require('crypto');

const machineCode = process.argv[2];
const secret = process.env.CASH_LEDGER_VENDOR_SECRET || 'CHANGE_THIS_VENDOR_SECRET_BEFORE_RELEASE_2026';

if (!machineCode) {
  console.error('Usage: node tools/generate-license-key.cjs MACHINE-CODE');
  process.exit(1);
}

const hash = crypto
  .createHash('sha256')
  .update(secret)
  .update('|')
  .update(machineCode.trim().toUpperCase())
  .digest('hex')
  .toUpperCase();

console.log(`${hash.slice(0, 8)}-${hash.slice(8, 16)}-${hash.slice(16, 24)}-${hash.slice(24, 32)}`);
