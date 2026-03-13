export const AUDIO_FILES = [
    'alert-01.aac',
    'alert-02.aac',
    'alert-03.aac',
    'alert-04.aac',
    'alert-05.aac',
    'alert-06.aac',
    'alert-07.aac',
    'alert-08.aac',
    'alert-09.aac',
    'alert-10.aac',
    'bip-bop-01.aac',
    'bip-bop-02.aac',
    'bip-bop-03.aac',
    'bip-bop-04.aac',
    'bip-bop-05.aac',
    'bip-bop-06.aac',
    'bip-bop-07.aac',
    'bip-bop-08.aac',
    'bip-bop-09.aac',
    'bip-bop-10.aac',
    'nope-01.aac',
    'nope-02.aac',
    'nope-03.aac',
    'nope-04.aac',
    'nope-05.aac',
    'nope-06.aac',
    'nope-07.aac',
    'nope-08.aac',
    'nope-09.aac',
    'nope-10.aac',
    'nope-11.aac',
    'nope-12.aac',
    'staplebops-01.aac',
    'staplebops-02.aac',
    'staplebops-03.aac',
    'staplebops-04.aac',
    'staplebops-05.aac',
    'staplebops-06.aac',
    'staplebops-07.aac',
    'yup-01.aac',
    'yup-02.aac',
    'yup-03.aac',
    'yup-04.aac',
    'yup-05.aac',
    'yup-06.aac',
] as const;

export type AudioSound = (typeof AUDIO_FILES)[number];

type AudioCategory = 'alert' | 'nope' | 'yup' | 'bip-bop' | 'staplebops';

export function categoryToDefaultSound(category: AudioCategory): AudioSound {
    return `${category}-01.aac` as AudioSound;
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
