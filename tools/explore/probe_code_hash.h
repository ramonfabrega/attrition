/* FNV-1a-64 compatibility fingerprint, not an authentication mechanism.
 * Capsule manifests separately bind the complete installed image with SHA-256.
 * Unsigned overflow is intentional; no original executable bytes live here.
 */
static unsigned long long probe_code_hash(const unsigned char *bytes, unsigned length) {
    unsigned long long hash = 0xcbf29ce484222325ULL;
    for (unsigned i = 0; i < length; i++) {
        hash ^= bytes[i];
        hash *= 0x100000001b3ULL;
    }
    return hash;
}
