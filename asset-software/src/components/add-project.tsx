"use client";
import {FormEvent, useState} from "react";
import { invoke } from '@tauri-apps/api/core';

export default function AddProjet() {
    const [name, setName] = useState("");

    const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        let name = e.target[0].value
        await invoke('add_projet', {name: name});
    };

    return (
        <>
            <div className=" bg-grey">

                { (
                    <form onSubmit={handleSubmit} className="flex">
                        <input
                            className="flex-1 w-42 ms-8 mt-1"
                            type="text"
                            placeholder="Nouveau projet"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            required
                        />
                            <button type="submit" className="flex-1 w-12 ms-2 cursor-pointer">+</button>
                    </form>
                )}
            </div>
        </>
    );
}
