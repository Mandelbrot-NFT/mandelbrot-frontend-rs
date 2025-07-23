use leptos::prelude::*;

#[component]
pub fn Guide() -> impl IntoView {
    view! {
        <div
            id="content"
            class="p-6 max-w-3xl mx-auto text-gray-800 space-y-6 overflow-y-auto max-h-[80vh] scroll-smooth"
        >
            <h1 class="text-3xl font-bold">"Welcome to Mandelbrot NFT!"</h1>

            <p>
                "This decentralized application allows you to interact with the Mandelbrot NFT ecosystem and create, trade, and
                explore unique visual representations of the Mandelbrot set. This guide will walk you through the main functionalities
                and steps to get started."
            </p>

            <h2 class="text-2xl font-semibold">"Prerequisites:"</h2>
            <ol class="list-decimal list-inside space-y-1">
                <li>"You will need a compatible web browser with a web3 wallet extension (such as MetaMask) installed."</li>
                <li>"Make sure you have some wOM tokens in your web3 wallet to pay for minting NFTs."</li>
            </ol>

            <h2 class="text-2xl font-semibold">"Getting Started:"</h2>

            <h3 class="text-xl font-semibold">"Step 1: Accessing Mandelbrot NFT"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"Open your web browser and navigate to https://mandelbrot-nft.onrender.com."</li>
                <li>"Ensure that your web3 wallet extension is active and connected to the Sepolia network."</li>
            </ol>

            <h3 class="text-xl font-semibold">"Step 2: Connect your Wallet"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"On the dApp interface, click the 'Connect Wallet' button."</li>
                <li>"Follow the prompts from your web3 wallet extension to connect it to the dApp."</li>
            </ol>

            <h3 class="text-xl font-semibold">"Step 3: Buy wrapper OM"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"On the dApp interface, click the 'Buy wOM' button."</li>
                <li>"You will be redirected to the Uniswap pair where you will have an opportunity to buy wOM."</li>
            </ol>

            <h3 class="text-xl font-semibold">"Step 4: Unwrap wrapper OM"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"On the dApp interface, select the amount of wOM that you wish to unwrap and click the 'Unwrap' button."</li>
                <li>"Once the transaction succeeds your balance will be refreshed."</li>
            </ol>

            <h3 class="text-xl font-semibold">"Step 5: Explore the Mandelbrot Set"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"You can pan and zoom on the Cartesian plane to explore different regions of the Mandelbrot set."</li>
                <li>"Each red or blue frame represents an NFT, and you can double click on any NFT to view its details."</li>
            </ol>

            <h3 class="text-xl font-semibold">"Step 6: Minting NFTs"</h3>
            <ol class="list-decimal list-inside space-y-1">
                <li>"To mint an NFT within the coordinates of an existing NFT, double click on the NFT of interest."</li>
                <li>"On the NFT details page, click the 'Bid' button."</li>
                <li>"Set the amount of OM you are willing to spend on the minting process."</li>
                <li>"Set the minimum bid amount needed for others to mint NFTs inside of your's."</li>
                <li>"Afterwards submit your bid, it will be represented as a yellow frame."</li>
                <li>"The owner of the parent NFT will review the bids and decide which NFTs get minted."</li>
            </ol>
        </div>
    }
}
