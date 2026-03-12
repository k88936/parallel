/** Shared Tailwind utility strings for dialog forms. */
export const df = {
    form: 'p-4',
    title: 'block text-base font-semibold mb-4',
    group: 'flex items-start mb-4',
    label: 'w-[100px] pr-3 pt-[6px] text-[13px] text-[var(--ring-text-color,#fff)]',
    control: 'flex-1 relative',
    input: 'w-full px-[10px] py-[6px] text-[13px] border border-[var(--ring-border-color,#3d3d3d)] rounded-[3px] bg-[var(--ring-input-background,#1e1e1e)] text-[var(--ring-text-color,#fff)] box-border focus:outline-none focus:border-[var(--ring-focused-border-color,#4a90d9)] disabled:opacity-50 disabled:cursor-not-allowed',
    inputM: 'max-w-[300px]',
    inputError: '!border-[var(--ring-error-color,#f00)]',
    errorBubble: 'absolute top-full left-0 mt-1 px-2 py-1 text-xs text-white bg-[var(--ring-error-color,#f00)] rounded-[3px] whitespace-nowrap z-10',
    footer: 'flex justify-end gap-2 mt-5 pt-4 border-t border-[var(--ring-border-color,#3d3d3d)]',
} as const;
