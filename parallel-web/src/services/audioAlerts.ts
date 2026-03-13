export type AudioCategory = 'alert' | 'nope' | 'yup' | 'bip-bop' | 'staplebops';

const AUDIO_CATEGORY_COUNTS: Record<AudioCategory, number> = {
    alert: 10,
    'bip-bop': 10,
    nope: 12,
    yup: 6,
    staplebops: 7,
};

export function getRandomAudioFile(category: AudioCategory): string {
    const count = AUDIO_CATEGORY_COUNTS[category];
    const randomIndex = Math.floor(Math.random() * count) + 1;
    return `${category}-${String(randomIndex).padStart(2, '0')}.aac`;
}

export function getAudioPath(fileName: string): string {
    return new URL(`../assets/audio/${fileName}`, import.meta.url).href;
}

export async function playAudioFile(filePath: string, volume: number): Promise<void> {
    return new Promise((resolve, reject) => {
        try {
            const audio = new Audio(filePath);
            audio.volume = Math.max(0, Math.min(1, volume));
            audio.onended = () => resolve();
            audio.onerror = () => reject(new Error('Failed to play audio file'));
            audio.play().catch(reject);
        } catch (error) {
            reject(error);
        }
    });
}
