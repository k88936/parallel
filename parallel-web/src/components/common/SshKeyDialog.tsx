import {useState, useEffect} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import type {SshKeyConfig} from '../../types';
import './DialogForm.css';

interface SshKeyDialogProps {
    show: boolean;
    onClose: () => void;
    onSubmit: (data: SshKeyConfig) => Promise<void>;
    initialData?: SshKeyConfig | null;
}

export const SshKeyDialog = ({show, onClose, onSubmit, initialData}: SshKeyDialogProps) => {
    const [name, setName] = useState('');
    const [key, setKey] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const isEdit = !!initialData;

    useEffect(() => {
        if (show) {
            if (initialData) {
                setName(initialData.name);
                setKey(initialData.key);
            } else {
                setName('');
                setKey('');
            }
            setError(null);
            setLoading(false);
        }
    }, [show, initialData]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) {
            setError('Name is required');
            return;
        }
        if (!key.trim()) {
            setError('SSH key is required');
            return;
        }
        setLoading(true);
        setError(null);
        try {
            await onSubmit({
                name: name.trim(),
                key: key.trim(),
            });
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to save SSH key');
        } finally {
            setLoading(false);
        }
    };

    const handleClose = () => {
        if (!loading) {
            onClose();
        }
    };

    return (
        <Dialog
            show={show}
            label={isEdit ? 'Edit SSH Key' : 'Add SSH Key'}
            onCloseAttempt={handleClose}
            onOverlayClick={handleClose}
            onEscPress={handleClose}
            showCloseButton
            trapFocus
            dense
        >
            <div>
                <form className="ring-form" onSubmit={handleSubmit}>
                    <span className="ring-form__title">{isEdit ? 'Edit SSH Key' : 'Add SSH Key'}</span>

                    <div className="ring-form__group">
                        <label htmlFor="ssh-name" className="ring-form__label">
                            Name
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="ssh-name"
                                className={`ring-input ring-input-size_m ${error && !name.trim() ? 'ring-input_error' : ''}`}
                                type="text"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="Enter key name"
                                disabled={loading}
                                autoFocus
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="ssh-key" className="ring-form__label">
                            Key
                        </label>
                        <div className="ring-form__control">
                            <textarea
                                id="ssh-key"
                                className={`ring-input ${error && !key.trim() ? 'ring-input_error' : ''}`}
                                value={key}
                                onChange={(e) => setKey(e.target.value)}
                                placeholder="Paste SSH private key"
                                disabled={loading}
                                rows={6}
                                style={{resize: 'vertical', maxWidth: '100%'}}
                            />
                            {error && <div className="ring-error-bubble active">{error}</div>}
                        </div>
                    </div>

                    <div className="ring-form__footer">
                        <Button primary type="submit" disabled={loading}>
                            {loading ? 'Saving...' : (isEdit ? 'Save' : 'Add')}
                        </Button>
                        <Button onClick={handleClose} disabled={loading}>
                            Cancel
                        </Button>
                    </div>
                </form>
            </div>
        </Dialog>
    );
};
