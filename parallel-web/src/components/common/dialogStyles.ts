/** Shared Tailwind utility strings for dialog forms. */
export const df = {
    form: 'p-4',
    group: 'flex items-start mb-4',
    label: 'w-[100px] pr-3 pt-[6px] text-[13px] text-[var(--ring-text-color,#fff)]',
    control: 'flex-1 relative',
    input: 'w-full px-[10px] py-[6px] text-[13px] border rounded-[3px] box-border focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed',
    inputM: 'max-w-[300px]',
    inputError: '!border-[var(--ring-error-color,#f00)]',
    errorBubble: 'absolute top-full left-0 mt-1 px-2 py-1 text-xs text-white rounded-[3px] whitespace-nowrap z-10',
} as const;
