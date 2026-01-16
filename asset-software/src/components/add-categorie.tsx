"use client";
import {FormEvent, useState} from "react";
import { invoke } from '@tauri-apps/api/core';

export default function AddCategory() {
    const [name, setName] = useState("");

    const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        let name = e.target[0].value
        await invoke('add_category', {name: name});
    };

    return (
        <>
            <div className=" bg-grey">

                { (
                    <form onSubmit={handleSubmit} className="flex">
                        <input
                            className="flex-1 w-42 ms-8"
                            type="text"
                            placeholder="Nouvelle catégorie"
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
