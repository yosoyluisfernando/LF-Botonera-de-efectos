const { pathToFileURL } = require('url');

/**
 * Gestor de audio encapsulado (Thin Client Architecture).
 * Este módulo actúa como intermediario para el futuro motor en Rust.
 * Por ahora utiliza la API de HTML5 de forma eficiente con limpieza de memoria.
 */
class AudioEngine {
    constructor() {
        // Utilizamos WeakMap para que si un botón o paleta se elimina, 
        // sus instancias de audio se limpien automáticamente por el Garbage Collector
        this.nodes = new WeakMap();
        this.prelistenAudio = null;
    }

    _getNode(btnInfo) {
        if (!this.nodes.has(btnInfo)) {
            this.nodes.set(btnInfo, { audio: null, clones: [] });
        }
        return this.nodes.get(btnInfo);
    }

    async play(btnInfo, filePath, options = {}) {
        const { volume = 1, loop = false, overlap = false, sinkId = '', onTimeUpdate, onEnded } = options;
        const node = this._getNode(btnInfo);

        if (node.audio && !node.audio.paused) {
            if (overlap) {
                // Modo overlap: Crear clon
                let clone = new Audio(pathToFileURL(filePath).href);
                clone.volume = volume;
                clone.loop = loop;
                node.clones.push(clone);
                
                await this._enrutarAudio(clone, sinkId);

                clone.onended = () => {
                    node.clones = node.clones.filter(c => c !== clone);
                    this._limpiarMemoriaAudio(clone);
                    if (onEnded && node.audio.paused && node.clones.length === 0) {
                        onEnded();
                    }
                };
                clone.play().catch(e => console.error("[AudioEngine] Error reproduciendo clon:", e));
                return clone;
            } else {
                this.stop(btnInfo);
            }
        }

        if (!node.audio) {
            node.audio = new Audio(pathToFileURL(filePath).href);
        }
        
        const audio = node.audio;
        audio.volume = volume;
        audio.loop = loop;
        
        await this._enrutarAudio(audio, sinkId);

        if (onTimeUpdate) {
            audio.ontimeupdate = () => onTimeUpdate(audio.currentTime, audio.duration);
        }

        audio.onended = () => {
            if (!loop && node.clones.length === 0) {
                this.stop(btnInfo);
            }
            if (onEnded) onEnded();
        };

        audio.play().catch(e => console.error("[AudioEngine] Error reproduciendo:", e));
        return audio;
    }

    stop(btnInfo) {
        const node = this.nodes.get(btnInfo);
        if (!node) return;

        if (node.audio) {
            node.audio.pause();
            node.audio.currentTime = 0;
            // Opcional: _limpiarMemoriaAudio(node.audio) y node.audio = null 
            // pero lo reutilizamos para no crear tantos objetos.
        }

        if (node.clones && node.clones.length > 0) {
            node.clones.forEach(c => {
                c.pause();
                c.currentTime = 0;
                this._limpiarMemoriaAudio(c);
            });
            node.clones = [];
        }
    }

    setVolume(btnInfo, volume) {
        const node = this.nodes.get(btnInfo);
        if (node && node.audio) {
            node.audio.volume = volume;
        }
    }

    setLoop(btnInfo, loop) {
        const node = this.nodes.get(btnInfo);
        if (node && node.audio) {
            node.audio.loop = loop;
        }
    }

    isPlaying(btnInfo) {
        const node = this.nodes.get(btnInfo);
        if (!node) return false;
        return (node.audio && !node.audio.paused) || (node.clones && node.clones.length > 0);
    }

    getTime(btnInfo) {
        const node = this.nodes.get(btnInfo);
        if (node && node.audio && node.audio.duration && !node.audio.paused) {
            return { currentTime: node.audio.currentTime, duration: node.audio.duration };
        }
        return null;
    }

    limpiarBoton(btnInfo) {
        this.stop(btnInfo);
        const node = this.nodes.get(btnInfo);
        if (node && node.audio) {
            this._limpiarMemoriaAudio(node.audio);
            node.audio = null;
        }
        this.nodes.delete(btnInfo);
    }

    // -- Prelisten --
    async playPrelisten(filePath, volume, sinkId, onTimeUpdate, onEnded) {
        this.stopPrelisten();
        if (!filePath) return;
        
        this.prelistenAudio = new Audio(pathToFileURL(filePath).href);
        this.prelistenAudio.volume = volume;
        await this._enrutarAudio(this.prelistenAudio, sinkId);
        
        if (onTimeUpdate) {
            this.prelistenAudio.ontimeupdate = () => {
               if (this.prelistenAudio.duration) {
                   onTimeUpdate(this.prelistenAudio.currentTime, this.prelistenAudio.duration);
               }
            };
        }
        
        this.prelistenAudio.onended = () => {
            this.stopPrelisten();
            if (onEnded) onEnded();
        };
        
        this.prelistenAudio.play().catch(e => console.error("[AudioEngine] Error prelisten:", e));
    }
    
    stopPrelisten() {
        if (this.prelistenAudio) {
            this.prelistenAudio.pause();
            this._limpiarMemoriaAudio(this.prelistenAudio);
            this.prelistenAudio = null;
        }
    }
    
    setPrelistenVolume(volume) {
        if (this.prelistenAudio) {
            this.prelistenAudio.volume = volume;
        }
    }

    // -- Funciones Internas --
    async _enrutarAudio(audioElement, sinkId) {
        if (!audioElement || typeof audioElement.setSinkId !== 'function') return;
        try {
            const realSinkId = (sinkId === 'default' || sinkId === 'global') ? '' : sinkId;
            await audioElement.setSinkId(realSinkId);
        } catch (e) {
            console.warn("[AudioEngine] Error al enrutar:", e);
        }
    }

    _limpiarMemoriaAudio(audioElement) {
        // Remover el source ayuda al recolector de basura (Garbage Collection)
        audioElement.removeAttribute('src'); 
        audioElement.load();
    }
}

module.exports = new AudioEngine();
