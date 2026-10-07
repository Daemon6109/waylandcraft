package dev.evvie.waylandcraft.bridge;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.stream.Stream;

import org.jetbrains.annotations.Nullable;
import org.jspecify.annotations.NonNull;
import org.lwjgl.system.Platform;

import com.mojang.blaze3d.opengl.GlDevice;
import com.mojang.blaze3d.systems.GpuDeviceBackend;
import com.mojang.blaze3d.systems.RenderSystem;

import dev.evvie.waylandcraft.WaylandCraftCommon;
import dev.evvie.waylandcraft.bridge.dmabuf.DmabufFeedbackData;
import dev.evvie.waylandcraft.bridge.dmabuf.DmabufFormat;
import dev.evvie.waylandcraft.desktop.RawDesktopEntry;
import dev.evvie.waylandcraft.egl.EGL;
import dev.evvie.waylandcraft.egl.EGLHelper;
import dev.evvie.waylandcraft.render.WindowFramebuffer;
import dev.evvie.waylandcraft.utils.CursorShape;
import net.minecraft.util.profiling.Profiler;
import net.minecraft.util.profiling.ProfilerFiller;

public class WaylandCraftBridge {
	
	private long instance;
	private ArrayList<WLCToplevel> toplevels = new ArrayList<WLCToplevel>();
	private ArrayList<WLCPopup> popups = new ArrayList<WLCPopup>();
	private ArrayList<WLCSurface> surfaces = new ArrayList<WLCSurface>();
	
	public IconSurface dndIcon = null;
	
	private LinkedList<WLCToplevel> focusOrder = new LinkedList<WLCToplevel>();
	
	private ArrayList<WLCToplevel> newToplevels = new ArrayList<WLCToplevel>();
	
	private @Nullable Integer lastMoveRequestSerial = null;
	private @Nullable ResizeRequest lastResizeRequest = null;
	
	static {
		boolean loaded = false;
		InputStream inputStream = openNativeLibraryFromJar();
		if(inputStream != null) {
			try {
				byte[] data = inputStream.readAllBytes();
				inputStream.close();
				
				File temp = File.createTempFile("waylandcraft-", "-libwaylandcraft.so");
				temp.deleteOnExit();
				
				FileOutputStream outputStream = new FileOutputStream(temp);
				outputStream.write(data);
				outputStream.close();
				
				System.load(temp.getAbsolutePath());
				loaded = true;
				
				WaylandCraftCommon.LOGGER.info("Loaded native library from jar");
			} catch (IOException e) {
				e.printStackTrace();
			}
		}
		
		if(!loaded) {
			WaylandCraftCommon.LOGGER.info("Native library could not be loaded from jar. Attempting to load from system");
			System.loadLibrary("waylandcraft");
		}
	}
	
	private static InputStream loadResource(String path) {
		WaylandCraftCommon.LOGGER.info("Looking for '" + path + "'...");
		return WaylandCraftBridge.class.getResourceAsStream(path);
	}
	
	private static InputStream openNativeLibraryFromJar() {
		InputStream stream = null;
		
		/* Attempt to load manually built native library */
		stream = loadResource("/libwaylandcraft.so");
		if(stream != null) return stream;
		
		/* Attempt to load from release library path */
		String arch;
		switch(Platform.getArchitecture()) {
		case X64: arch = "x86_64"; break;
		case ARM64: arch = "arm64"; break;
		default: arch = null; break;
		}
		
		if(arch != null) {
			String platform = "linux-gnu-" + arch;
			stream = loadResource("/libwaylandcraft-" + platform + ".so");
			if(stream != null) return stream;
		}
		
		return null;
	}
	
	private WaylandCraftBridge(long instance) {
		/* Constructor used by the native code.
		 * DO NOT USE ANY BRIDGE FUNCTIONS HERE! The instance pointer is uninitialized until
		 * WaylandCraftBridge#init() returns!
		 */
		this.instance = instance;
	}
	
	public static WaylandCraftBridge start() {
		DmabufFeedbackData dmabufFeedbackData = initBackend();
		WaylandCraftBridge bridge = init(dmabufFeedbackData);
		
		// Add shutdown thread to clean up resources on normal exit
		Runtime.getRuntime().addShutdownHook(new Thread(bridge::shutdownHook));
		
		return bridge;
	}
	
	private static DmabufFeedbackData initBackend() {
		GpuDeviceBackend deviceBackend = RenderSystem.getDevice().backend;
		if(deviceBackend instanceof GlDevice) {
			return initBackendEGL();
		}
		
		WaylandCraftCommon.LOGGER.error("Unsupported graphics backend!");
		return null;
	}
	
	private static DmabufFeedbackData initBackendEGL() {
		long eglDisplay = EGL.getEGLDisplay();
		if(eglDisplay == 0) {
			throw new RuntimeException("Failed to get EGL display!");
		}
		
		String renderNodePath = EGLHelper.queryRenderNodePath(eglDisplay);
		if(renderNodePath == null) {
			WaylandCraftCommon.LOGGER.error("Failed to query for drm render node! This could indicate a software renderer. Disabling dmabuf functionality.");
			return null;
		}
		
		DmabufFormat[] formats = EGLHelper.queryDmabufFormats(eglDisplay).toArray(DmabufFormat[]::new);
		long device = drmDeviceByPath(renderNodePath);
		
		return new DmabufFeedbackData(device, formats);
	}
	
	private void shutdownHook() {
		shutdown(instance);
		instance = 0;
	}
	
	public void update() {
		ProfilerFiller profiler = Profiler.get();
		profiler.push("wayland");
		
		// Dispatch wayland client events
		profiler.push("dispatch clients");
		dispatchClients(instance);
		profiler.pop();
		
		// Add newly mapped toplevels to newToplevels
		for(WLCToplevel toplevel : toplevels) {
			boolean mapped = toplevel.isMapped();
			if(mapped && !toplevel.wasMapped) {
				newToplevels.add(toplevel);
			}
			toplevel.wasMapped = mapped;
		}
		
		updateFocusOrder();
		
		// Do client frame callbacks
		for(WLCSurface surface : surfaces) {
			surface.sendFrame();
		}
		
		// Flush outgoing display buffers
		flushDisplay(instance);
		
		WindowFramebuffer.endFrame();
		
		profiler.pop();
	}
	
	protected void addSurface(WLCSurface surface) {
		surfaces.add(surface);
	}
	
	protected void deleteSurface(WLCSurface surface) {
		surfaces.remove(surface);
		surface.destroy();
	}
	
	protected void addToplevel(WLCToplevel toplevel) {
		toplevels.add(toplevel);
	}
	
	protected void deleteToplevel(WLCToplevel toplevel) {
		toplevels.remove(toplevel);
	}
	
	public WLCSurface[] getAllSurfaces() {
		return surfaces.toArray(WLCSurface[]::new);
	}
	
	public WLCToplevel[] getNewToplevels() {
		WLCToplevel[] toplevels = newToplevels.toArray(WLCToplevel[]::new);
		newToplevels.clear();
		
		return toplevels;
	}
	
	/*
	protected WLCToplevel getOrCreateToplevel(long topLevelHandle) {
		for(WLCToplevel toplevel : toplevels) {
			if(toplevel.getHandle() == topLevelHandle) return toplevel;
		}
		WLCToplevel toplevel = new WLCToplevel(topLevelHandle);
		
		long surfaceHandle = toplevelSurface(this.instance, topLevelHandle);
		WLCSurface surface = getOrCreateSurface(surfaceHandle);
		toplevel.surface = surface;
		
		toplevels.add(toplevel);
		return toplevel;
	}
	
	public WLCToplevel[] getNewToplevels() {
		WLCToplevel[] toplevels = newToplevels.toArray(WLCToplevel[]::new);
		newToplevels.clear();
		
		return toplevels;
	}
	
	protected WLCPopup getOrCreatePopup(long handle) {
		for(WLCPopup popup : popups) {
			if(popup.getHandle() == handle) return popup;
		}
		WLCPopup popup = new WLCPopup(handle);
		
		long surfaceHandle = popupSurface(this.instance, handle);
		WLCSurface surface = getOrCreateSurface(surfaceHandle);
		popup.surface = surface;
		
		popup.parentHandle = popupParent(this.instance, handle);
		
		popups.add(popup);
		return popup;
	}
	
	protected WLCSurface getOrCreateSurface(long handle) {
		for(WLCSurface surface : surfaces) {
			if(surface.getHandle() == handle) return surface;
		}
		WLCSurface surface = new WLCSurface(handle);
		surfaces.add(surface);
		return surface;
	}
	
	protected DmabufTexture getDmabuf(long handle) {
		for(DmabufTexture dmabuf : dmabufs) {
			if(dmabuf.handle == handle) return dmabuf;
		}
		return null;
	}
	
	private void deleteNonExistingToplevels(long[] remainingHandles) {
		ArrayList<WLCToplevel> toplevels_new = new ArrayList<WLCToplevel>();
		for(WLCToplevel toplevel : this.toplevels) {
			if(ArrayUtils.contains(remainingHandles, toplevel.getHandle())) {
				toplevels_new.add(toplevel);
			}
			else {
				freeToplevel(this.instance, toplevel.takeHandle());
			}
		}
		this.toplevels = toplevels_new;
	}
	
	private void deleteNonExistingPopups(long[] remainingHandles) {
		ArrayList<WLCPopup> popups_new = new ArrayList<WLCPopup>();
		for(WLCPopup popup : this.popups) {
			if(ArrayUtils.contains(remainingHandles, popup.getHandle())) {
				popups_new.add(popup);
			}
			else {
				freePopup(this.instance, popup.takeHandle());
			}
		}
		this.popups = popups_new;
	}
	
	protected boolean importDmabuf(Dmabuf dmabuf) {
		try {
			DmabufTexture texture = BufferTexture.createDmabufTexture(dmabuf);
			dmabufs.add(texture);
			return true;
		} catch(DmabufImportFailedException e) {
			return false;
		}
	}
	
	private void updateDmabufs() {
		checkImportDmabuf(instance);
		
		long[] remainingHandles = dmabufs(instance);
		ArrayList<DmabufTexture> dmabufs_new = new ArrayList<DmabufTexture>();
		for(DmabufTexture dmabuf : this.dmabufs) {
			// If the dmabuf texture is not attached to a real wl_buffer anymore, free the imported resources
			boolean retained = ArrayUtils.contains(remainingHandles, dmabuf.handle);
			if(!retained) dmabuf.doFree();
			
			// Remove it from the list and free the texture if no longer attached to any surface
			boolean used = false;
			for(WLCSurface surface : surfaces) {
				if(surface.getBuffer() == dmabuf) {
					used = true;
					break;
				}
			}
			if(retained || used) {
				dmabufs_new.add(dmabuf);
			}
			else {
				dmabuf.doReleaseTexure();
			}
		}
		this.dmabufs = dmabufs_new;
	}
	
	private void deleteUnvisitedSurfaces() {
		ArrayList<WLCSurface> surfaces_new = new ArrayList<WLCSurface>();
		for(WLCSurface surface : this.surfaces) {
			if(surface.visited) {
				surfaces_new.add(surface);
			}
			else {
				surface.destroy();
				freeSurface(this.instance, surface.takeHandle());
			}
		}
		this.surfaces = surfaces_new;
	}
	
	private void findPopupParent(WLCPopup popup) {
		// Popups cannot change their parent, so if one is found, it's the one
		if(popup.parent != null) return;
		
		for(WLCToplevel toplevel : toplevels) {
			if(toplevel.getHandle() == popup.parentHandle) {
				popup.parent = toplevel;
				return;
			}
		}
		
		for(WLCPopup popup2 : popups) {
			if(popup2.getHandle() == popup.parentHandle) {
				popup.parent = popup2;
				return;
			}
		}
	}
	
	public void update() {
		ProfilerFiller profiler = Profiler.get();
		profiler.push("wayland");
		
		// Dispatch wayland client events
		profiler.push("dispatch clients");
		dispatchClients(instance);
		profiler.pop();
		
		// Find all available toplevels and delete ones that no longer exist
		long[] toplevelHandles = toplevels(instance);
		deleteNonExistingToplevels(toplevelHandles);
		
		// Find all available popups and delete ones that no longer exist
		long[] popupHandles = popups(instance);
		deleteNonExistingPopups(popupHandles);
		
		long[] minimizeRequests = minimizeReq(instance);
		long[] maximizeRequests = maximizeReq(instance);
		long[] unmaximizeRequests = unmaximizeReq(instance);
		long[] fullscreenRequests = fullscreenReq(instance);
		long[] unfullscreenRequests = unfullscreenReq(instance);
		long[] fullscreened = fullscreened(instance);
		
		int[] moveRequest = moveRequest(instance);
		if(moveRequest != null) {
			lastMoveRequestSerial = moveRequest[0];
		}
		
		int[] resizeRequest = resizeRequest(instance);
		if(resizeRequest != null) {
			lastResizeRequest = new ResizeRequest(resizeRequest[0], resizeRequest[1]);
		}
		
		// Reset surface visited state
		for(WLCSurface surface : surfaces) {
			surface.visited = false;
		}
		
		profiler.push("update surface tree");
		// Create new toplevels when necessary
		// Update surface tree geometry and properties of all toplevels
		for(long handle : toplevelHandles) {
			WLCToplevel toplevel = getOrCreateToplevel(handle);
			WLCSurface root = toplevel.getSurfaceTree();
			toplevel.lastChild = updateSurfaceTree(this.instance, root);
			
			updateGeometry(toplevel);
			toplevel.title = toplevelTitle(toplevel.getHandle());
			toplevel.appID = toplevelAppID(toplevel.getHandle());
			
			if(ArrayUtils.contains(minimizeRequests, handle)) toplevel.requests.minimize = true;
			if(ArrayUtils.contains(maximizeRequests, handle)) toplevel.requests.maximize= true;
			if(ArrayUtils.contains(unmaximizeRequests, handle)) toplevel.requests.unmaximize = true;
			if(ArrayUtils.contains(fullscreenRequests, handle)) toplevel.requests.fullscreen = true;
			if(ArrayUtils.contains(unfullscreenRequests, handle)) toplevel.requests.unfullscreen = true;
			
			toplevel.fullscreen = ArrayUtils.contains(fullscreened, handle);
		}
		
		// Create new popups when necessary
		// Update surface tree geometry, parent relationships and offsets of all popups
		for(long handle : popupHandles) {
			WLCPopup popup = getOrCreatePopup(handle);
			findPopupParent(popup);
			
			int[] offset = popupOffset(handle);
			popup.offsetX = offset[0];
			popup.offsetY = offset[1];
			
			WLCSurface root = popup.getSurfaceTree();
			popup.lastChild = updateSurfaceTree(this.instance, root);
			updateGeometry(popup);
		}
		
		long dndIconHandle = dndIcon(instance);
		if(dndIconHandle != 0) {
			WLCSurface dndIconSurface = getOrCreateSurface(dndIconHandle);
			if(dndIcon != null && dndIcon.surface != dndIconSurface) dndIcon = null;
			if(dndIcon == null) dndIcon = new IconSurface(dndIconSurface);
			
			updateSurfaceData(instance, dndIcon.surface);
			dndIcon.surface.visited = true;
		}
		else {
			dndIcon = null;
		}
		
		// All surface trees have now been walked. Now delete all unvisited surfaces
		deleteUnvisitedSurfaces();
		profiler.pop();
		
		// Resolve surface parent handles to actual surfaces
		for(WLCSurface surface : surfaces) {
			if(surface.parentHandle != 0) {
				surface.parent = getOrCreateSurface(surface.parentHandle);
			}
			else {
				surface.parent = null;
			}
		}
		
		List<WLCAbstractWindow> allWindows = Stream.of(toplevels, popups).flatMap((l) -> l.stream()).collect(Collectors.toList());
		
		profiler.push("update surface data");
		// Update all surface buffers
		for(WLCAbstractWindow window : allWindows) {
			WLCSurface root = window.getSurfaceTree();
			for(WLCSurface surface = root; surface != null; surface = surface.getNextChild()) {
				updateSurfaceData(instance, surface);
				calculateSubpos(surface);
			}
		}
		profiler.pop();
		
		for(WLCToplevel toplevel : toplevels) {
			boolean mapped = toplevel.isMapped();
			if(mapped && !toplevel.wasMapped) {
				newToplevels.add(toplevel);
			}
			toplevel.wasMapped = mapped;
		}
		
		profiler.push("framebuffer");
		updateFramebuffers();
		profiler.pop();
		
		updateDmabufs();
		
		updateFocusOrder();
		
		// Do client frame callbacks
		for(WLCSurface surface : surfaces) {
			sendFrame(surface.getHandle());
		}
		
		// Flush outgoing display buffers
		flushDisplay(instance);
		
		profiler.pop();
	}
	
	private void updateFramebuffers() {
		List<WLCAbstractWindow> allWindows = Stream.of(toplevels, popups).flatMap((l) -> l.stream()).collect(Collectors.toList());
		
		// Render windows
		for(WLCAbstractWindow window : allWindows) {
			if(window.framebuffer == null) {
				window.framebuffer = new WindowFramebuffer(window.getSurfaceTree());
				framebuffers.add(window.framebuffer);
			}
			window.framebuffer.render();
		}
		
		// Render dnd icon
		if(dndIcon != null) {
			if(dndIcon.framebuffer == null) {
				dndIcon.framebuffer = new WindowFramebuffer(dndIcon.surface);
				framebuffers.add(dndIcon.framebuffer);
			}
			dndIcon.framebuffer.render();
		}
		
		// Cleanup unused framebuffers
		ArrayList<WindowFramebuffer> usedFramebuffers = new ArrayList<WindowFramebuffer>();
		for(WindowFramebuffer framebuffer : framebuffers) {
			if(framebuffer.surfaceTree.isAlive()) {
				usedFramebuffers.add(framebuffer);
			}
			else {
				framebuffer.destroy();
			}
		}
		framebuffers.retainAll(usedFramebuffers);
		
		WindowFramebuffer.endFrame();
	}
	*/
	
//	private void updateGeometry(WLCAbstractWindow window) {
//		int[] data = surfaceXDGGeometry(window.surface.getHandle());
//		SurfaceGeometry geometry;
//		
//		if(data == null) {
//			geometry = new SurfaceGeometry(0, 0, window.surface.width(), window.surface.height());
//		}
//		else {
//			geometry = new SurfaceGeometry(data[0], data[1], data[2], data[3]);
//		}
//		
//		window.geometry = geometry;
//	}
	
	public WLCToplevel[] getToplevels() {
		return toplevels.toArray(new WLCToplevel[toplevels.size()]);
	}
	
	public WLCToplevel[] getMappedToplevels() {
		return toplevels.stream().filter((t) -> t.isMapped()).toArray(WLCToplevel[]::new);
	}
	
	public WLCToplevel getToplevel(long handle) {
		return toplevels.stream().filter((w) -> w.getHandle() == handle).findAny().orElse(null);
	}
	
	public WLCPopup[] getPopups() {
		return popups.toArray(new WLCPopup[popups.size()]);
	}
	
	public WLCPopup[] getMappedPopups() {
		return popups.stream().filter((t) -> t.isMapped()).toArray(WLCPopup[]::new);
	}
	
	public String getSocket() {
		return socket(this.instance);
	}
	
	public @Nullable String getX11Display() {
		return x11Display(this.instance);
	}
	
	// Create pointer motion event
	public void sendMotion(double x, double y) {
		pointerMotion(instance, x, y);
	}
	
	// Create pointer motion event
	public void sendMotionRefocus(@Nullable WLCSurface surface, double x, double y) {
		pointerMotionFocus(instance, surface, x, y);
	}
	
	// Send relative pointer motion to surface with pointer focus
	public void sendRelativeMotion(double dx, double dy) {
		pointerRelMotion(instance, dx, dy);
	}
	
	// Remove pointer focus from all surfaces
	public void sendMotionOutside() {
		pointerLeave(instance);
	}
	
	// Check if there is an active pointer lock on the surface and lock the pointer if yes
	public boolean maybeLockPointer(@NonNull WLCSurface surface) {
		return maybePointerLock(instance, surface);
	}
	
	// Drop an active pointer lock (if any)
	public void unlockPointer() {
		pointerUnlock(instance);
	}
	
	// Create pointer button event. `button` has to be the linux button code, state is 1 for pressed, 0 for released
	public int sendButton(int button, int state) {
		return pointerButton(instance, button, state);
	}
	
	// Create pointer axis event. `axis` is the scroll axis (0 for vertical, 1 for horizontal)
	public void sendScroll(int axis, double value) {
		pointerAxis(instance, axis, value);
	}
	
	// Get active cursor shape
	public CursorShape getCursorShape() {
		return CursorShape.fromId(cursorShape(instance));
	}
	
	// Set keyboard focus to a toplevel
	public void focusSurface(@Nullable WLCToplevel toplevel) {
		keyboardFocus(instance, toplevel);
		
		// Make toplevel most recently focused
		if(toplevel != null) {
			focusOrder.remove(toplevel);
			focusOrder.addLast(toplevel);
		}
	}
	
	// Mark keyboard as active, forward any pressed modifiers and pressed keys, etc. to the clients
	public void activateKeyboard() {
		keyboardActivate(instance);
	}
	
	// Mark keyboard as inactive, don't forward any keyboard state to the clients
	public void deactivateKeyboard() {
		keyboardDeactivate(instance);
	}
	
	private void updateFocusOrder() {
		focusOrder.removeIf((t) -> !toplevels.contains(t));
		for(WLCToplevel toplevel : toplevels) {
			if(!focusOrder.contains(toplevel)) focusOrder.addLast(toplevel);
		}
	}
	
	// Find the most recently focused toplevel that exists
	public WLCToplevel getMostRecentFocus() {
		updateFocusOrder();
		return focusOrder.peekLast();
	}
	
	// Find the most recently focused toplevel that exists
	public Stream<WLCToplevel> getMostToLeastRecentFocus() {
		updateFocusOrder();
		return focusOrder.reversed().stream();
	}
	
	public void pressKey(int scancode) {
		keyboardInput(instance, scancode, 1);
	}
	
	public void releaseKey(int scancode) {
		keyboardInput(instance, scancode, 0);
	}
	
	// Update internal key state
	public void internalKeyUpdate(int scancode, boolean pressed) {
		keyboardUpdate(instance, scancode, pressed);
	}
	
	public void resizeToplevelInteractive(WLCToplevel toplevel, int width, int height) {
//		toplevelResize(toplevel.getHandle(), width, height, true);
	}
	
	public void resizeToplevel(WLCToplevel toplevel, int width, int height) {
//		toplevelResize(toplevel.getHandle(), width, height, false);
	}
	
	public void resizeToplevelOverride(WLCToplevel toplevel, int width, int height) {
//		toplevelResizeOvr(toplevel.getHandle(), width, height);
	}
	
	public void maximizeToplevel(WLCToplevel toplevel) {
//		toplevelMaximize(instance, toplevel.getHandle());
	}
	
	public void fullscreenToplevel(WLCToplevel toplevel) {
//		toplevelFullscreen(instance, toplevel.getHandle());
	}
	
	public Integer checkMoveRequest() {
//		if(lastMoveRequestSerial == null) return null;
//		int serial = lastMoveRequestSerial.intValue();
//		lastMoveRequestSerial = null;
//		return serial;
		return null;
	}
	
	public ResizeRequest checkResizeRequest() {
//		if(lastResizeRequest == null) return null;
//		ResizeRequest req = lastResizeRequest;
//		lastResizeRequest = null;
//		return req;
		return null;
	}
	
	public void resizeOutput(int width, int height) {
//		outputResize(instance, width, height);
	}
	
	public void setOutputBounds(int width, int height) {
//		outputSetBounds(instance, width, height);
	}
	
	public Size getOutputSize() {
//		int[] size = outputSize(instance);
//		return new Size(size[0], size[1]);
		return new Size(1, 1);
	}
	
	public Size getOutputBounds() {
//		int[] size = outputBounds(instance);
//		return new Size(size[0], size[1]);
		return new Size(1, 1);
	}
	
	public RawDesktopEntry loadDesktopEntry(File path) {
//		return loadDesktopEntry(instance, path.getAbsolutePath());
		return null;
	}
	
	public RawDesktopEntry[] loadSystemDesktopEntries() {
//		return loadDesktopEntries(instance);
		return new RawDesktopEntry[] {};
	}
	
	public boolean renderSVG(File file, int width, int height, long bufferPtr) {
//		return renderSVG(file.getAbsolutePath(), width, height, bufferPtr);
		return false;
	}
	
	public boolean execApp(String appId) {
//		return execApp(instance, appId);
		return true;
	}
	
	public void setPreferredTerminal(String cmd) {
//		setPreferredTerminal(instance, cmd);
	}
	
	public boolean setKeymapFromStr(String keymap) {
		return setKeymapFromStr(instance, keymap);
	}
	
	public Integer checkDndRequest() {
//		int[] serial = checkDndRequest(instance);
//		if(serial == null) return null;
//		return serial[0];
		return null;
	}
	
	public void dndCancel() {
//		dndCancel(instance);
	}
	
	public void dndDrop() {
//		dndDrop(instance);
	}
	
	public void sendDndMotion(WLCSurface surface, double x, double y) {
//		long handle = surface == null ? 0 : surface.getHandle();
//		dndMotion(instance, handle, x, y);
	}
	
	public static record Size(int width, int height) {}
	
	public static record ResizeRequest(int serial, int edges) {}
	
	/* Additional bridge native functions
	 * 
	 * Some native methods are directly on various objects like WLCSurface#checkInputRegion, ...
	 * Some functionality is implemented directly in native code and calls java code
	 * The remaining stuff is here:
	 */
	
	/* General bridge functions */
	private static native WaylandCraftBridge init(@Nullable DmabufFeedbackData dmabufFeedbackData);
	private static native void shutdown(long instance);
	private static native void dispatchClients(long instance);
	private static native void flushDisplay(long instance);
	private static native String socket(long instance);
	private static native String x11Display(long instance);
	
	/* Direct Rendering Manager functionality */
	private static native long drmDeviceByPath(String path);
	private static native long drmDeviceByMajorMinor(int major, int minor);
	
	/* Seat functionality */
	private static native void pointerMotion(long instance, double x, double y);
	private static native void pointerMotionFocus(long instance, @Nullable WLCSurface surface, double x, double y);
	private static native void pointerRelMotion(long instance, double dx, double dy);
	private static native boolean maybePointerLock(long instance, @NonNull WLCSurface surface);
	private static native void pointerUnlock(long instance);
	private static native void pointerLeave(long instance);
	private static native int pointerButton(long instance, int button, int state);
	private static native void pointerAxis(long instance, int axis, double value);
	private static native int cursorShape(long instance);
	private static native void keyboardFocus(long instance, @Nullable WLCToplevel toplevel);
	private static native void keyboardActivate(long instance);
	private static native void keyboardDeactivate(long instance);
	private static native void keyboardInput(long instance, int scancode, int action);
	private static native void keyboardUpdate(long instance, int scancode, boolean pressed);
	private static native boolean setKeymapFromStr(long instance, String keymap);
	
	
	// TODO: Implement the following stuff (or alternatives to them):
	/*
	// Resize toplevel
	private static native void toplevelResize(long topLevelHandle, int width, int height, boolean interactive);
	// Resize toplevel override, keep maximized and fullscreen state, stop interactive resize
	private static native void toplevelResizeOvr(long topLevelHandle, int width, int height);
	
	// Collect all toplevels that have sent a minimize request and clear the list
	private static native long[] minimizeReq(long instance);
	// Collect all toplevels that have sent a maximize request and clear the list
	private static native long[] maximizeReq(long instance);
	// Collect all toplevels that have sent an unmaximize request and clear the list
	private static native long[] unmaximizeReq(long instance);
	// Collect all toplevels that have sent a fullscreen request and clear the list
	private static native long[] fullscreenReq(long instance);
	// Collect all toplevels that have sent an unfullscreen request and clear the list
	private static native long[] unfullscreenReq(long instance);
	
	// Collect up to one serial of a sent interactive move request
	private static native int[] moveRequest(long instance);
	// Collect up to one serial of a sent interactive resize request
	private static native int[] resizeRequest(long instance);
	
	// All toplevels that are currently in fullscreen
	private static native long[] fullscreened(long instance);
	
	private static native void toplevelMaximize(long instance, long topLevelHandle);
	private static native void toplevelFullscreen(long instance, long topLevelHandle);
	
	private static native long[] popups(long instance);
	private static native long popupSurface(long instance, long topLevelHandle);
	// Query the parent of a popup
	// Returned handle is a handle either to a toplevel or another popup
	private static native long popupParent(long instance, long topLevelHandle);
	// Query popup local offset coordinates
	// Returns two-element list containing x,y
	private static native int[] popupOffset(long popupHandle);
	
	private static native int[] outputSize(long instance);
	private static native int[] outputBounds(long instance);
	
	// Update virtual output dimensions
	private static native void outputResize(long instance, int width, int height);
	
	// Update virtual output maximum window bounds
	private static native void outputSetBounds(long instance, int width, int height);
	
	private static native RawDesktopEntry loadDesktopEntry(long instance, String path);
	private static native RawDesktopEntry[] loadDesktopEntries(long instance);
	
	private static native boolean renderSVG(String path, int width, int height, long bufferPtr);
	
	private static native boolean execApp(long instance, String appId);
	private static native void setPreferredTerminal(long instance, String cmd);
	
	private static native int[] checkDndRequest(long instance);
	private static native boolean checkDndActive(long instance);
	private static native void dndCancel(long instance);
	private static native void dndDrop(long instance);
	private static native void dndMotion(long instance, long surfaceHandle, double x, double y);
	private static native long dndIcon(long instance);
	*/
	
}
