"use client";
import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';

export default function AddProjet() {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");

    const handleSubmit = async (e) => {
        e.preventDefault();
        let name = e.target[0].value
        await invoke('add_project', {name: name});
    };

    return (
        <>
            <div className="flex bg-grey">
                <nav className="navbar mr-2">
                    <button onClick={() => setOpen(true)} className="add-btn">+</button>
                </nav>

                {open && (
                    <div className="modal-backdrop">
                        <div className="modal">
                            <form onSubmit={handleSubmit}>
                                <input
                                    type="text"
                                    placeholder="Nom du projet"
                                    value={name}
                                    onChange={(e) => setName(e.target.value)}
                                    required
                                />
                                    <button type="submit">✔️</button>
                                    <button type="button" onClick={() => setOpen(false)}>
                                        ❌
                                    </button>
                            </form>
                        </div>
                    </div>
                )}
            </div>
        </>
    );
}
