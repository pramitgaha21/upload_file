const CRC32 = require("crc-32");

function updateChecksum(chunk, checksum) {

  // Calculate the signed checksum for the given chunk
  const signedChecksum = CRC32.buf(Buffer.from(chunk, "binary"), 0);

  // Convert the signed checksum to an unsigned value
  const unsignedChecksum = signedChecksum >>> 0;

  // Return the updated checksum
  return unsignedChecksum;
}

module.exports = {
  updateChecksum,
};