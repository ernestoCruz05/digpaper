import { openDB } from 'idb';

const DB_NAME = 'charta-offline-db';
const DB_VERSION = 2;
const STORE_NAME = 'offline-uploads';
const CACHE_STORE = 'message-cache';

export const initDB = async () => {
  return openDB(DB_NAME, DB_VERSION, {
    upgrade(db) {
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'id', autoIncrement: true });
      }
      if (!db.objectStoreNames.contains(CACHE_STORE)) {
        db.createObjectStore(CACHE_STORE, { keyPath: 'key' });
      }
    },
  });
};

export const saveUploadToQueue = async (file, originalName, audioBlob = null, projectId = null) => {
  const db = await initDB();
  return db.add(STORE_NAME, {
    file,
    originalName,
    audioBlob,
    projectId,
    timestamp: Date.now(),
  });
};

export const getUploadQueue = async () => {
  const db = await initDB();
  return db.getAll(STORE_NAME);
};

export const removeFromQueue = async (id) => {
  const db = await initDB();
  return db.delete(STORE_NAME, id);
};

// ── Message cache ──
export const cacheMessages = async (projectId, messages) => {
  try {
    const db = await initDB();
    await db.put(CACHE_STORE, { key: `forum-${projectId}`, data: messages, ts: Date.now() });
  } catch {}
};

export const getCachedMessages = async (projectId) => {
  try {
    const db = await initDB();
    const entry = await db.get(CACHE_STORE, `forum-${projectId}`);
    return entry?.data || null;
  } catch { return null; }
};
