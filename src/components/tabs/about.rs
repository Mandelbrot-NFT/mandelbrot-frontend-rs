use leptos::prelude::*;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <div class="flex flex-col max-w-4xl mx-auto p-4 space-y-6">
            <div class="aspect-w-16 aspect-h-9 w-full">
                <iframe
                    class="w-full h-full rounded-lg shadow"
                    src="https://www.youtube.com/embed/OlD2rcm971U"
                    title="YouTube video player"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    allowfullscreen=true
                />
            </div>

            <div
                id="content"
                class="p-6 bg-white rounded-lg shadow-md max-h-[60vh] overflow-y-auto space-y-4 text-gray-800 leading-relaxed"
            >
                <p>"The Mandelbrot set is a mathematical concept that can be visualized on a Cartesian plane with coordinates ranging from -2 to 2 on both the x and y axes."</p>

                <p>"In our system, each NFT (Non-Fungible Token) represents a specific region within these coordinates, defined by a rectangular shape. We have developed custom software that generates a graphical representation for each NFT."</p>

                <p>"The original NFT, called the Origin NFT, is owned by the project DAO (Decentralized Autonomous Organisation) and covers the entire coordinate range of -2 to 2 on both axes."</p>

                <p>"Every NFT has the ability to create a fixed number (20 in this case) of child NFTs within its own boundaries. This hierarchical structure allows us to trace back each NFT to the Origin NFT."</p>

                <p>"Creating a new NFT requires the use of a cryptocurrency token called OM. The number of NFTs that can be minted within a parent NFT is limited."</p>

                <p>"To determine which NFTs get minted, users submit bids, and the owner of the parent NFT selects the winning bids."</p>

                <p>"When an NFT is successfully minted, the OM used for minting is distributed among all the parent NFTs, and a portion of it is locked within a newly minted NFT."</p>

                <p>"The owner of a parent NFT can set a minimum amount of OM that must be used in a mint bid. All child NFTs created within that parent NFT must adhere to this minimum requirement."</p>

                <p>"Owner of an NFT can burn it, given that it doesn't have any child NFTs in it. When an NFT is burned, OM that was locked inside of it is transferred to its owner."</p>
            </div>
        </div>
    }
}
