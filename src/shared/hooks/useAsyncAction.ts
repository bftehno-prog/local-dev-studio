import { useCallback, useState } from 'react';

export function useAsyncAction() {
  const [message, setMessage] = useState('');

  const showError = useCallback((error: unknown) => {
    setMessage(error instanceof Error ? error.message : String(error));
  }, []);

  const run = useCallback(
    async (action: () => Promise<unknown>, success: string, after?: () => Promise<unknown>) => {
      try {
        setMessage('');
        await action();
        setMessage(success);
        await after?.();
      } catch (error) {
        showError(error);
      }
    },
    [showError],
  );

  return { message, setMessage, showError, run };
}
