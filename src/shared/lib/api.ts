import { api } from '../api/commands';

export type AppError = {
  code?: string;
  message: string;
  details?: string;
  hint?: string;
};

export function normalizeApiError(error: unknown): AppError {
  if (typeof error === 'string') {
    return { message: error };
  }

  if (error instanceof Error) {
    return { message: error.message };
  }

  if (error && typeof error === 'object' && 'message' in error) {
    return { message: String((error as { message: unknown }).message) };
  }

  return { message: 'Unknown error' };
}

export { api };
