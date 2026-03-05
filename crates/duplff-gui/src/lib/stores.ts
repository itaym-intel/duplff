import { writable } from 'svelte/store';
import type { Screen, ScanConfig, DuplicateReport, SortMode } from './types';
import { defaultConfig } from './types';

export const currentScreen = writable<Screen>('setup');
export const scanConfig = writable<ScanConfig>(defaultConfig());
export const report = writable<DuplicateReport | null>(null);
export const focusedGroup = writable<number>(0);
export const expandedGroups = writable<Set<number>>(new Set());
export const markedFiles = writable<Set<string>>(new Set());
export const filterText = writable<string>('');
export const sortMode = writable<SortMode>('wasted');
export const ignoredGroups = writable<Set<number>>(new Set());
