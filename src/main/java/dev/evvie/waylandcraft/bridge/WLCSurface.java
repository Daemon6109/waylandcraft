package dev.evvie.waylandcraft.bridge;

import java.util.ArrayList;
import java.util.List;

import org.jetbrains.annotations.Nullable;

import dev.evvie.waylandcraft.render.BufferTexture;
import dev.evvie.waylandcraft.render.BufferTexture.DmabufTexture;
import net.minecraft.util.Mth;

public class WLCSurface {
	
	protected long handle = 0;
	protected boolean dirty = false;
	
	@Nullable
	private BufferTexture buffer = null;
	
	@Nullable
	protected WLCSurface parent = null;
	
	protected WLCSurface[] children = new WLCSurface[0];
	
	// Entire surface tree in drawing order (back to front). Contains the surface itself. This field is non-null exactly when this surface is a root surface.
	@Nullable
	protected WLCSurface[] surfaceDrawTree = null;
	
	// Entire surface tree in input order (front to back). Contains the surface itself. This field is non-null exactly when this surface is a root surface.
	@Nullable
	protected WLCSurface[] surfaceInputTree = null;
	
	// Surface size. By default the size of the attached buffer.
	private int width = 0;
	private int height = 0;
	
	@Nullable
	private ViewportSource sourceView = null;
	
	// X and Y offsets relative to parent coords
	protected int xoff = 0;
	protected int yoff = 0;
	
	// Total calculated offsets
	public int xSubpos = 0;
	public int ySubpos = 0;
	
	private ArrayList<SurfaceDamage> surfaceDamage = new ArrayList<>();
	private ArrayList<BufferDamage> bufferDamage = new ArrayList<>();
	
	private WLCSurface(long handle) {
		this.handle = handle;
	}
	
	public boolean isAlive() {
		return handle != 0;
	}
	
	protected void destroy() {
		if(buffer != null) buffer.release();
	}
	
	// Attach a shared memory buffer
	// The surface width and height are reset to the given buffer dimensions.
	protected void attachShmBuffer(long ptr, int width, int height, int format, int stride) {
		removeBuffer();
		
		this.buffer = BufferTexture.createShmTexture(ptr, width, height, format, stride);
		this.width = width;
		this.height = height;
	}
	
	// Attach a single pixel buffer
	// The surface width and height are reset to 1.
	protected void attachSinglePixelBuffer(byte r, byte g, byte b, byte a) {
		removeBuffer();
		
		this.buffer = BufferTexture.createSinglePixelTexture(r, g, b, a);
		this.width = 1;
		this.height = 1;
	}
	
	// Attach an already known dmabuf
	// The surface width and height are reset to the given buffer dimensions.
	// Returns false if no DmabufTexture by that handle was found.
	protected boolean attachDmabuf(long handle) {
		removeBuffer();
		
//		DmabufTexture dmabuf = WaylandCraft.instance.bridge.getDmabuf(handle);
		DmabufTexture dmabuf = null;
		if(dmabuf == null) return false;
		
		this.buffer = dmabuf;
		this.width = buffer.width;
		this.height = buffer.height;
		
		dmabuf.copyData();
		return true;
	}
	
	protected void removeBuffer() {
		if(buffer != null) buffer.release();
		this.buffer = null;
		this.width = this.height = 0;
	}
	
	// Set viewport source dimensions
	// Crops the surface to the specified rectangle.
	protected void setViewportSrc(double x, double y, double width, double height) {
		this.sourceView = new ViewportSource(x, y, width, height);
		this.width = (int) width;
		this.height = (int) height;
	}
	
	protected void unsetViewportSrc() {
		this.sourceView = null;
	}
	
	// Set viewport destination dimensions
	// Overrides this surfaces width & height values.
	protected void setViewportDst(int width, int height) {
		this.width = width;
		this.height = height;
	}
	
	protected void clearDamage() {
		surfaceDamage.clear();
		bufferDamage.clear();
	}
	
	protected void addSurfaceDamage(int x, int y, int width, int height) {
		if(buffer == null) return;
		
		this.surfaceDamage.add(new SurfaceDamage(x, y, width, height));
		
		double sourceX = 0;
		double sourceY = 0;
		double sourceWidth = buffer.width;
		double sourceHeight = buffer.height;
		if(sourceView != null) {
			sourceX = sourceView.x;
			sourceY = sourceView.y;
			sourceWidth = sourceView.width;
			sourceHeight = sourceView.height;
		}
		
		double bx = sourceX + x / (double) this.width * sourceWidth;
		double by = sourceY + y / (double) this.height * sourceHeight;
		double bw = width / (double) this.width * sourceWidth;
		double bh = height / (double) this.height * sourceHeight;
		
		this.bufferDamage.add(new BufferDamage(Mth.floor(bx), Mth.floor(by), Mth.ceil(bw), Mth.ceil(bh)));
	}
	
	protected void addBufferDamage(int x, int y, int width, int height) {
		if(buffer == null) return;
		
		bufferDamage.add(new BufferDamage(x, y, width, height));
		
		double sx = x;
		double sy = y;
		double sw = width;
		double sh = height;
		
		double sourceWidth = buffer.width;
		double sourceHeight = buffer.height;
		if(sourceView != null) {
			sx -= sourceView.x;
			sy -= sourceView.y;
			sourceWidth = sourceView.width;
			sourceHeight = sourceView.height;
		}
		
		sx *= this.width / sourceWidth;
		sy *= this.height / sourceHeight;
		sw *= this.width / sourceWidth;
		sh *= this.height / sourceHeight;
		
		this.surfaceDamage.add(new SurfaceDamage(Mth.floor(sx), Mth.floor(sy), Mth.ceil(sw), Mth.ceil(sh)));
	}
	
	public List<SurfaceDamage> getSurfaceDamage() {
		return surfaceDamage;
	}
	
	public List<BufferDamage> getBufferDamage() {
		return bufferDamage;
	}
	
	public int width() {
		return width;
	}
	
	public int height() {
		return height;
	}
	
	public ViewportSource getViewportSource() {
		return sourceView;
	}
	
	@Nullable
	public BufferTexture getBuffer() {
		return this.buffer;
	}
	
	@Nullable
	public WLCSurface getParent() {
		return this.parent;
	}
	
	public WLCSurface[] getChildren() {
		return this.children;
	}
	
	@Nullable
	public WLCSurface[] getDrawTree() {
		return surfaceDrawTree;
	}
	
	@Nullable
	public WLCSurface[] getInputTree() {
		return surfaceInputTree;
	}
	
	public native void sendFrame();
	
	// Surface-local dimensions of the source rectangle in a buffer
	public static final record ViewportSource(double x, double y, double width, double height) {
	}
	
	// Surface-local region describing contents damage
	public static final record SurfaceDamage(int x, int y, int width, int height) {
	}
	
	// Buffer-local region describing contents damage
	public static final record BufferDamage(int x, int y, int width, int height) {
	}
	
}
